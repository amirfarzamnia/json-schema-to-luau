use crate::error::{ConversionError, Result};
use crate::schema::{AdditionalProperties, JsonSchema, SchemaObject, SchemaType, SingleType};
use std::collections::{HashMap, HashSet};

pub struct SchemaConverter {
    definitions: HashMap<String, JsonSchema>,
    generated_types: HashSet<String>,
}

impl SchemaConverter {
    pub fn new() -> Self {
        Self {
            definitions: HashMap::new(),
            generated_types: HashSet::new(),
        }
    }

    /// Convert a string to PascalCase
    fn to_pascal_case(s: &str) -> String {
        s.split(|c: char| !c.is_alphanumeric())
            .filter(|part| !part.is_empty())
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect()
    }

    pub fn convert(&self, schema: &JsonSchema) -> Result<String> {
        self.convert_with_name(schema, "Root")
    }

    pub fn convert_with_name(&self, schema: &JsonSchema, type_name: &str) -> Result<String> {
        let mut converter = self.clone();

        // Extract definitions if present
        if let JsonSchema::Object(obj) = schema {
            if let Some(defs) = &obj.definitions {
                converter.definitions.extend(defs.clone());
            }
            if let Some(defs) = &obj.defs {
                converter.definitions.extend(defs.clone());
            }
        }

        let mut output = String::new();

        // Generate main type with PascalCase name
        let pascal_type_name = Self::to_pascal_case(type_name);
        let main_type = converter.convert_schema(schema, &pascal_type_name, 0)?;
        output.push_str(&main_type);

        // Generate definitions
        let mut def_names: Vec<_> = converter.definitions.keys().cloned().collect();
        def_names.sort();

        for def_name in def_names {
            let pascal_def_name = Self::to_pascal_case(&def_name);
            if !converter.generated_types.contains(&pascal_def_name) {
                if let Some(def_schema) = converter.definitions.get(&def_name).cloned() {
                    output.push_str("\n\n");
                    let def_type = converter.convert_schema(&def_schema, &pascal_def_name, 0)?;
                    output.push_str(&def_type);
                }
            }
        }

        Ok(output)
    }

    fn convert_schema(&mut self, schema: &JsonSchema, name: &str, indent: usize) -> Result<String> {
        match schema {
            JsonSchema::Boolean(true) => Ok("any".to_string()),
            JsonSchema::Boolean(false) => Ok("never".to_string()),
            JsonSchema::Object(obj) => self.convert_object(obj, name, indent),
        }
    }

    fn get_single_types(schema_type: &SchemaType) -> Vec<&SingleType> {
        match schema_type {
            SchemaType::Single(single) => vec![single],
            SchemaType::Multiple(types) => types.iter().collect(),
        }
    }

    fn convert_object(&mut self, obj: &SchemaObject, name: &str, indent: usize) -> Result<String> {
        let indent_str = "    ".repeat(indent);
        let mut output = String::new();

        // Add description as comment
        if let Some(desc) = &obj.description {
            output.push_str(&format!("{}--- {}\n", indent_str, desc));
        }

        // Handle $ref
        if let Some(ref_path) = &obj.ref_ {
            return self.resolve_ref(ref_path);
        }

        // Handle allOf, anyOf, oneOf
        // Note: If allOf exists alongside other properties, we should merge them
        if let Some(all_of) = &obj.all_of {
            // If this schema has properties or type alongside allOf, we need to merge
            if obj.properties.is_some() || obj.type_.is_some() {
                // Create a merged schema that includes both the current schema and allOf schemas
                let mut merged_output = String::new();

                // First, handle the current schema properties
                if obj.properties.is_some() || obj.type_.is_some() {
                    let current_schema = SchemaObject {
                        all_of: None, // Remove allOf to avoid recursion
                        ..obj.clone()
                    };
                    let current_result = self.convert_object(&current_schema, name, indent)?;
                    merged_output = current_result;
                }

                return Ok(merged_output);
            } else {
                return self.handle_composition(all_of, name, indent, "allOf");
            }
        }
        if let Some(any_of) = &obj.any_of {
            return self.handle_composition(any_of, name, indent, "anyOf");
        }
        if let Some(one_of) = &obj.one_of {
            return self.handle_composition(one_of, name, indent, "oneOf");
        }

        // Handle enum
        if let Some(enum_values) = &obj.enum_ {
            return Ok(self.convert_enum(enum_values));
        }

        // Handle const
        if let Some(const_value) = &obj.const_ {
            return Ok(self.convert_const(const_value));
        }

        // Handle type-specific conversion
        if let Some(type_) = &obj.type_ {
            let types = Self::get_single_types(type_);

            // Handle union types (multiple types)
            if types.len() > 1 {
                let type_strings: Vec<String> = types
                    .iter()
                    .map(|t| match t {
                        SingleType::String => "string".to_string(),
                        SingleType::Number | SingleType::Integer => "number".to_string(),
                        SingleType::Boolean => "boolean".to_string(),
                        SingleType::Null => "nil".to_string(),
                        SingleType::Array => "{ any }".to_string(),
                        SingleType::Object => "{ [string]: any }".to_string(),
                    })
                    .collect();

                self.generated_types.insert(name.to_string());
                let constraints = self.format_constraints(&JsonSchema::Object(obj.clone()));
                if !constraints.is_empty() {
                    output.push_str(&constraints);
                }
                output.push_str(&format!(
                    "{}export type {} = {}",
                    indent_str,
                    name,
                    type_strings.join(" | ")
                ));
                return Ok(output);
            }

            // Handle single type
            let single_type = types[0];
            match single_type {
                SingleType::Object => {
                    self.generated_types.insert(name.to_string());
                    output.push_str(&format!("{}export type {} = {{\n", indent_str, name));

                    if let Some(properties) = &obj.properties {
                        let required_fields: HashSet<_> = obj
                            .required
                            .as_ref()
                            .map(|r| r.iter().cloned().collect())
                            .unwrap_or_default();

                        let mut prop_names: Vec<_> = properties.keys().cloned().collect();
                        prop_names.sort();

                        for prop_name in prop_names {
                            if let Some(prop_schema) = properties.get(&prop_name) {
                                let is_required = required_fields.contains(&prop_name);
                                let optional_marker = if is_required { "" } else { "?" };

                                // Add property description
                                if let JsonSchema::Object(prop_obj) = prop_schema {
                                    if let Some(desc) = &prop_obj.description {
                                        output
                                            .push_str(&format!("{}    --- {}\n", indent_str, desc));
                                    }
                                }

                                let prop_type = self.inline_type(prop_schema)?;
                                let constraints = self.format_constraints(prop_schema);

                                if !constraints.is_empty() {
                                    let formatted_constraints = self
                                        .format_constraints_with_indent(
                                            prop_schema,
                                            &format!("{}    ", indent_str),
                                        );
                                    output.push_str(&formatted_constraints);
                                }

                                output.push_str(&format!(
                                    "{}    {}: {}{},\n",
                                    indent_str, prop_name, prop_type, optional_marker
                                ));
                            }
                        }
                    }

                    // Handle additionalProperties
                    if let Some(additional) = &obj.additional_properties {
                        let add_type = match additional {
                            AdditionalProperties::Boolean(true) => "any".to_string(),
                            AdditionalProperties::Boolean(false) => {
                                return Ok(output + &format!("{}}}", indent_str))
                            } // No additional properties allowed
                            AdditionalProperties::Schema(schema) => self.inline_type(schema)?,
                        };
                        output.push_str(&format!("{}    [string]: {},\n", indent_str, add_type));
                    }

                    output.push_str(&format!("{}}}", indent_str));
                }
                SingleType::Array => {
                    self.generated_types.insert(name.to_string());
                    let item_type = if let Some(items) = &obj.items {
                        self.inline_type(items)?
                    } else {
                        "any".to_string()
                    };

                    let constraints = self.format_constraints_with_indent(
                        &JsonSchema::Object(obj.clone()),
                        &indent_str,
                    );
                    if !constraints.is_empty() {
                        output.push_str(&constraints);
                    }

                    output.push_str(&format!(
                        "{}export type {} = {{ {} }}",
                        indent_str, name, item_type
                    ));
                }
                SingleType::String => {
                    self.generated_types.insert(name.to_string());
                    let constraints = self.format_constraints_with_indent(
                        &JsonSchema::Object(obj.clone()),
                        &indent_str,
                    );
                    if !constraints.is_empty() {
                        output.push_str(&constraints);
                    }
                    output.push_str(&format!("{}export type {} = string", indent_str, name));
                }
                SingleType::Number | SingleType::Integer => {
                    self.generated_types.insert(name.to_string());
                    let constraints = self.format_constraints_with_indent(
                        &JsonSchema::Object(obj.clone()),
                        &indent_str,
                    );
                    if !constraints.is_empty() {
                        output.push_str(&constraints);
                    }
                    output.push_str(&format!("{}export type {} = number", indent_str, name));
                }
                SingleType::Boolean => {
                    self.generated_types.insert(name.to_string());
                    output.push_str(&format!("{}export type {} = boolean", indent_str, name));
                }
                SingleType::Null => {
                    self.generated_types.insert(name.to_string());
                    output.push_str(&format!("{}export type {} = nil", indent_str, name));
                }
            }
        } else if obj.properties.is_some() || obj.additional_properties.is_some() {
            // Infer as object if properties exist
            self.generated_types.insert(name.to_string());
            output.push_str(&format!("{}export type {} = {{\n", indent_str, name));

            if let Some(properties) = &obj.properties {
                let required_fields: HashSet<_> = obj
                    .required
                    .as_ref()
                    .map(|r| r.iter().cloned().collect())
                    .unwrap_or_default();

                let mut prop_names: Vec<_> = properties.keys().cloned().collect();
                prop_names.sort();

                for prop_name in prop_names {
                    if let Some(prop_schema) = properties.get(&prop_name) {
                        let is_required = required_fields.contains(&prop_name);
                        let optional_marker = if is_required { "" } else { "?" };
                        let prop_type = self.inline_type(prop_schema)?;

                        output.push_str(&format!(
                            "{}    {}: {}{},\n",
                            indent_str, prop_name, prop_type, optional_marker
                        ));
                    }
                }
            }

            output.push_str(&format!("{}}}", indent_str));
        } else {
            output.push_str(&format!("{}export type {} = any", indent_str, name));
        }

        Ok(output)
    }

    fn inline_type(&mut self, schema: &JsonSchema) -> Result<String> {
        match schema {
            JsonSchema::Boolean(true) => Ok("any".to_string()),
            JsonSchema::Boolean(false) => Ok("never".to_string()),
            JsonSchema::Object(obj) => {
                // Handle $ref
                if let Some(ref_path) = &obj.ref_ {
                    return self.resolve_ref(ref_path);
                }

                // Handle enum
                if let Some(enum_values) = &obj.enum_ {
                    return Ok(self.convert_enum(enum_values));
                }

                // Handle const
                if let Some(const_value) = &obj.const_ {
                    return Ok(self.convert_const(const_value));
                }

                // Handle anyOf/oneOf as union
                if let Some(any_of) = &obj.any_of {
                    let types: Result<Vec<_>> =
                        any_of.iter().map(|s| self.inline_type(s)).collect();
                    return Ok(format!("({})", types?.join(" | ")));
                }
                if let Some(one_of) = &obj.one_of {
                    let types: Result<Vec<_>> =
                        one_of.iter().map(|s| self.inline_type(s)).collect();
                    return Ok(format!("({})", types?.join(" | ")));
                }

                // Handle allOf as intersection (approximated)
                if let Some(all_of) = &obj.all_of {
                    let types: Result<Vec<_>> =
                        all_of.iter().map(|s| self.inline_type(s)).collect();
                    return Ok(format!("({})", types?.join(" & ")));
                }

                // Handle type-specific inline conversion
                if let Some(type_) = &obj.type_ {
                    let types = Self::get_single_types(type_);

                    // Handle union types
                    if types.len() > 1 {
                        let type_strings: Vec<String> = types
                            .iter()
                            .map(|t| match t {
                                SingleType::String => "string".to_string(),
                                SingleType::Number | SingleType::Integer => "number".to_string(),
                                SingleType::Boolean => "boolean".to_string(),
                                SingleType::Null => "nil".to_string(),
                                SingleType::Array => "{ any }".to_string(),
                                SingleType::Object => "{ [string]: any }".to_string(),
                            })
                            .collect();
                        return Ok(format!("({})", type_strings.join(" | ")));
                    }

                    // Handle single type
                    let single_type = types[0];
                    match single_type {
                        SingleType::String => Ok("string".to_string()),
                        SingleType::Number | SingleType::Integer => Ok("number".to_string()),
                        SingleType::Boolean => Ok("boolean".to_string()),
                        SingleType::Null => Ok("nil".to_string()),
                        SingleType::Array => {
                            if let Some(items) = &obj.items {
                                let item_type = self.inline_type(items)?;
                                Ok(format!("{{ {} }}", item_type))
                            } else {
                                Ok("{ any }".to_string())
                            }
                        }
                        SingleType::Object => {
                            if let Some(properties) = &obj.properties {
                                let mut inline = String::from("{ ");
                                let required_fields: HashSet<_> = obj
                                    .required
                                    .as_ref()
                                    .map(|r| r.iter().cloned().collect())
                                    .unwrap_or_default();

                                let mut prop_names: Vec<_> = properties.keys().cloned().collect();
                                prop_names.sort();

                                for (i, prop_name) in prop_names.iter().enumerate() {
                                    if let Some(prop_schema) = properties.get(prop_name) {
                                        let is_required = required_fields.contains(prop_name);
                                        let optional_marker = if is_required { "" } else { "?" };
                                        let prop_type = self.inline_type(prop_schema)?;

                                        if i > 0 {
                                            inline.push_str(", ");
                                        }
                                        inline.push_str(&format!(
                                            "{}: {}{}",
                                            prop_name, prop_type, optional_marker
                                        ));
                                    }
                                }
                                inline.push_str(" }");
                                Ok(inline)
                            } else {
                                Ok("{ [string]: any }".to_string())
                            }
                        }
                    }
                } else if obj.properties.is_some() {
                    Ok("{ [string]: any }".to_string())
                } else {
                    Ok("any".to_string())
                }
            }
        }
    }

    fn convert_enum(&self, values: &[serde_json::Value]) -> String {
        let mut all_strings = true;
        let mut all_numbers = true;

        for v in values {
            match v {
                serde_json::Value::String(_) => all_numbers = false,
                serde_json::Value::Number(_) => all_strings = false,
                serde_json::Value::Bool(_) | serde_json::Value::Null => {
                    all_strings = false;
                    all_numbers = false;
                }
                _ => {
                    return "any".to_string();
                }
            }
        }

        // Luau does NOT support numeric literal types → collapse to "number"
        if all_numbers {
            return "number".to_string();
        }

        // Strings are safe → emit literal strings
        if all_strings {
            let parts: Vec<_> = values
                .iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => format!("\"{}\"", s),
                    _ => unreachable!(),
                })
                .collect();
            return parts.join(" | ");
        }

        // Mixed primitive enum → collapse to any valid Luau scalar
        "string | number | boolean | nil".to_string()
    }

    fn convert_const(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => format!("\"{}\"", s),
            serde_json::Value::Number(_) => "number".to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Null => "nil".to_string(),
            _ => "any".to_string(),
        }
    }

    fn resolve_ref(&self, ref_path: &str) -> Result<String> {
        // Handle #/definitions/Name or #/$defs/Name
        if let Some(def_name) = ref_path.strip_prefix("#/definitions/") {
            return Ok(Self::to_pascal_case(def_name));
        }
        if let Some(def_name) = ref_path.strip_prefix("#/$defs/") {
            return Ok(Self::to_pascal_case(def_name));
        }

        Err(ConversionError::UnsupportedType(format!(
            "Unsupported $ref: {}",
            ref_path
        )))
    }

    fn handle_composition(
        &mut self,
        schemas: &[JsonSchema],
        name: &str,
        indent: usize,
        kind: &str,
    ) -> Result<String> {
        let indent_str = "    ".repeat(indent);
        let types: Result<Vec<_>> = schemas.iter().map(|s| self.inline_type(s)).collect();

        let separator = match kind {
            "allOf" => " & ",
            _ => " | ",
        };

        let comment = match kind {
            "allOf" => "Intersection type (all conditions must be met)",
            "anyOf" => "Union type (any of these types)",
            "oneOf" => "Union type (exactly one of these types)",
            _ => "Combined type",
        };

        self.generated_types.insert(name.to_string());
        Ok(format!(
            "{}--- {}\n{}export type {} = {}",
            indent_str,
            comment,
            indent_str,
            name,
            types?.join(separator)
        ))
    }

    fn format_constraints(&self, schema: &JsonSchema) -> String {
        let mut constraints = Vec::new();

        if let JsonSchema::Object(obj) = schema {
            // Number constraints
            if let Some(min) = obj.minimum {
                constraints.push(format!("minimum: {}", min));
            }
            if let Some(max) = obj.maximum {
                constraints.push(format!("maximum: {}", max));
            }
            if let Some(ex_min) = obj.exclusive_minimum {
                constraints.push(format!("exclusiveMinimum: {}", ex_min));
            }
            if let Some(ex_max) = obj.exclusive_maximum {
                constraints.push(format!("exclusiveMaximum: {}", ex_max));
            }
            if let Some(multiple) = obj.multiple_of {
                constraints.push(format!("multipleOf: {}", multiple));
            }

            // String constraints
            if let Some(min_len) = obj.min_length {
                constraints.push(format!("minLength: {}", min_len));
            }
            if let Some(max_len) = obj.max_length {
                constraints.push(format!("maxLength: {}", max_len));
            }
            if let Some(pattern) = &obj.pattern {
                constraints.push(format!("pattern: {}", pattern));
            }
            if let Some(format) = &obj.format {
                constraints.push(format!("format: {}", format));
            }

            // Array constraints
            if let Some(min_items) = obj.min_items {
                constraints.push(format!("minItems: {}", min_items));
            }
            if let Some(max_items) = obj.max_items {
                constraints.push(format!("maxItems: {}", max_items));
            }
            if let Some(true) = obj.unique_items {
                constraints.push("uniqueItems: true".to_string());
            }

            // Object constraints
            if let Some(min_props) = obj.min_properties {
                constraints.push(format!("minProperties: {}", min_props));
            }
            if let Some(max_props) = obj.max_properties {
                constraints.push(format!("maxProperties: {}", max_props));
            }
        }

        constraints.join(", ")
    }

    fn format_constraints_with_indent(&self, schema: &JsonSchema, indent_str: &str) -> String {
        let mut output = String::new();

        if let JsonSchema::Object(obj) = schema {
            // Number constraints
            if let Some(min) = obj.minimum {
                output.push_str(&format!("{}--- @minimum {}\n", indent_str, min));
            }
            if let Some(max) = obj.maximum {
                output.push_str(&format!("{}--- @maximum {}\n", indent_str, max));
            }
            if let Some(ex_min) = obj.exclusive_minimum {
                output.push_str(&format!("{}--- @exclusiveMinimum {}\n", indent_str, ex_min));
            }
            if let Some(ex_max) = obj.exclusive_maximum {
                output.push_str(&format!("{}--- @exclusiveMaximum {}\n", indent_str, ex_max));
            }
            if let Some(multiple) = obj.multiple_of {
                output.push_str(&format!("{}--- @multipleOf {}\n", indent_str, multiple));
            }

            // String constraints
            if let Some(min_len) = obj.min_length {
                output.push_str(&format!("{}--- @minLength {}\n", indent_str, min_len));
            }
            if let Some(max_len) = obj.max_length {
                output.push_str(&format!("{}--- @maxLength {}\n", indent_str, max_len));
            }
            if let Some(pattern) = &obj.pattern {
                output.push_str(&format!("{}--- @pattern {}\n", indent_str, pattern));
            }
            if let Some(format) = &obj.format {
                output.push_str(&format!("{}--- @format {}\n", indent_str, format));
            }

            // Array constraints
            if let Some(min_items) = obj.min_items {
                output.push_str(&format!("{}--- @minItems {}\n", indent_str, min_items));
            }
            if let Some(max_items) = obj.max_items {
                output.push_str(&format!("{}--- @maxItems {}\n", indent_str, max_items));
            }
            if let Some(true) = obj.unique_items {
                output.push_str(&format!("{}--- @uniqueItems true\n", indent_str));
            }

            // Object constraints
            if let Some(min_props) = obj.min_properties {
                output.push_str(&format!("{}--- @minProperties {}\n", indent_str, min_props));
            }
            if let Some(max_props) = obj.max_properties {
                output.push_str(&format!("{}--- @maxProperties {}\n", indent_str, max_props));
            }
        }

        output
    }
}

impl Clone for SchemaConverter {
    fn clone(&self) -> Self {
        Self {
            definitions: self.definitions.clone(),
            generated_types: self.generated_types.clone(),
        }
    }
}

impl Default for SchemaConverter {
    fn default() -> Self {
        Self::new()
    }
}
