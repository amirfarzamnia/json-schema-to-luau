use convert_case::{Case, Casing};
use std::collections::{HashMap, HashSet};

use crate::error::{ConversionError, Result};
use crate::schema::{AdditionalProperties, JsonSchema, SchemaObject, SchemaType, SingleType};

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

    pub fn convert(&self, schema: &JsonSchema) -> Result<String> {
        self.convert_with_name(schema, "Root")
    }

    pub fn convert_with_name(&self, schema: &JsonSchema, type_name: &str) -> Result<String> {
        let mut converter = self.clone();
        converter.extract_definitions(schema);

        let mut output = String::new();

        // Generate main type with PascalCase name
        let pascal_type_name = type_name.to_case(Case::Pascal);
        let main_type = converter.convert_schema(schema, &pascal_type_name, 0)?;
        output.push_str(&main_type);

        // Generate definitions
        converter.generate_definitions(&mut output)?;

        // Ensure exactly one newline at EOF
        if !output.ends_with('\n') {
            output.push('\n');
        }

        Ok(output)
    }

    fn extract_definitions(&mut self, schema: &JsonSchema) {
        if let JsonSchema::Object(obj) = schema {
            if let Some(defs) = &obj.definitions {
                self.definitions.extend(defs.clone());
            }
            if let Some(defs) = &obj.defs {
                self.definitions.extend(defs.clone());
            }
        }
    }

    fn generate_definitions(&mut self, output: &mut String) -> Result<()> {
        let mut def_names: Vec<_> = self.definitions.keys().cloned().collect();
        def_names.sort();

        for def_name in def_names {
            let pascal_def_name = def_name.to_case(Case::Pascal);
            if !self.generated_types.contains(&pascal_def_name) {
                if let Some(def_schema) = self.definitions.get(&def_name).cloned() {
                    output.push_str("\n\n");
                    let def_type = self.convert_schema(&def_schema, &pascal_def_name, 0)?;
                    output.push_str(&def_type);
                }
            }
        }

        Ok(())
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

        // Handle references
        if let Some(ref_path) = &obj.ref_ {
            return self.resolve_ref(ref_path);
        }

        // Handle composition types (allOf, anyOf, oneOf)
        if let Some(result) = self.handle_composition_types(obj, name, indent)? {
            return Ok(result);
        }

        // Handle enum
        if let Some(enum_values) = &obj.enum_ {
            self.generated_types.insert(name.to_string());
            let indent_str = "    ".repeat(indent);
            let union = self.convert_enum(enum_values);
            return Ok(format!("{}export type {} = {}", indent_str, name, union));
        }

        // Handle const
        if let Some(const_value) = &obj.const_ {
            self.generated_types.insert(name.to_string());
            let indent_str = "    ".repeat(indent);
            let literal = self.convert_const(const_value);
            return Ok(format!("{}export type {} = {}", indent_str, name, literal));
        }

        // Handle type-specific conversion
        self.handle_type_conversion(obj, name, indent, &mut output)?;

        Ok(output)
    }

    fn handle_composition_types(
        &mut self,
        obj: &SchemaObject,
        name: &str,
        indent: usize,
    ) -> Result<Option<String>> {
        // allOf handling
        if let Some(all_of) = &obj.all_of {
            let parent_has_props = obj.properties.is_some()
                || obj.additional_properties.is_some()
                || obj.required.is_some();

            // If parent schema has its own properties → MERGE (your documented behavior)
            if parent_has_props {
                let mut merged = obj.clone();
                merged.all_of = None;

                for sub in all_of {
                    if let JsonSchema::Object(sub_obj) = sub {
                        // merge properties
                        if let Some(sub_props) = &sub_obj.properties {
                            merged
                                .properties
                                .get_or_insert_with(Default::default)
                                .extend(sub_props.clone());
                        }

                        // merge required
                        if let Some(sub_req) = &sub_obj.required {
                            merged
                                .required
                                .get_or_insert_with(Default::default)
                                .extend(sub_req.clone());
                        }

                        // merge additionalProperties
                        if let Some(additional) = &sub_obj.additional_properties {
                            merged.additional_properties = Some(additional.clone());
                        }
                    }
                }

                return Ok(Some(self.convert_object(&merged, name, indent)?));
            }

            // No parent properties → INTERSECTION
            let types: Result<Vec<_>> = all_of.iter().map(|s| self.inline_type(s)).collect();
            let indent_str = "    ".repeat(indent);
            self.generated_types.insert(name.to_string());

            return Ok(Some(format!(
                "{}export type {} = ({})",
                indent_str,
                name,
                types?.join(" & ")
            )));
        }

        // anyOf / oneOf
        if let Some(any_of) = &obj.any_of {
            return Ok(Some(
                self.handle_composition(any_of, name, indent, "anyOf")?,
            ));
        }

        if let Some(one_of) = &obj.one_of {
            return Ok(Some(
                self.handle_composition(one_of, name, indent, "oneOf")?,
            ));
        }

        Ok(None)
    }

    fn handle_type_conversion(
        &mut self,
        obj: &SchemaObject,
        name: &str,
        indent: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent_str = "    ".repeat(indent);
        self.generated_types.insert(name.to_string());

        if let Some(type_) = &obj.type_ {
            let types = Self::get_single_types(type_);

            // Handle union types (multiple types)
            if types.len() > 1 {
                let type_strings = self.map_types_to_strings(&types);
                let constraints = self
                    .format_constraints_with_indent(&JsonSchema::Object(obj.clone()), &indent_str);
                if !constraints.is_empty() {
                    output.push_str(&constraints);
                }
                output.push_str(&format!(
                    "{}export type {} = {}",
                    indent_str,
                    name,
                    type_strings.join(" | ")
                ));
                return Ok(());
            }

            // Handle single type
            let single_type = types[0];
            match single_type {
                SingleType::Object => {
                    self.generate_object_type(obj, name, indent, output)?;
                }
                SingleType::Array => {
                    self.generate_array_type(obj, name, indent, output)?;
                }
                SingleType::String | SingleType::Number | SingleType::Integer => {
                    let type_name = match single_type {
                        SingleType::String => "string",
                        SingleType::Number | SingleType::Integer => "number",
                        _ => unreachable!(),
                    };
                    let constraints = self.format_constraints_with_indent(
                        &JsonSchema::Object(obj.clone()),
                        &indent_str,
                    );
                    if !constraints.is_empty() {
                        output.push_str(&constraints);
                    }
                    output.push_str(&format!(
                        "{}export type {} = {}",
                        indent_str, name, type_name
                    ));
                }
                SingleType::Boolean => {
                    output.push_str(&format!("{}export type {} = boolean", indent_str, name));
                }
                SingleType::Null => {
                    output.push_str(&format!("{}export type {} = nil", indent_str, name));
                }
            }
        } else if obj.properties.is_some() || obj.additional_properties.is_some() {
            // Infer as object if properties exist
            self.generate_object_type(obj, name, indent, output)?;
        } else {
            output.push_str(&format!("{}export type {} = any", indent_str, name));
        }

        Ok(())
    }

    fn map_types_to_strings(&self, types: &[&SingleType]) -> Vec<String> {
        types
            .iter()
            .map(|t| match t {
                SingleType::String => "string".to_string(),
                SingleType::Number | SingleType::Integer => "number".to_string(),
                SingleType::Boolean => "boolean".to_string(),
                SingleType::Null => "nil".to_string(),
                SingleType::Array => "{ any }".to_string(),
                SingleType::Object => "{ [string]: any }".to_string(),
            })
            .collect()
    }

    fn generate_object_type(
        &mut self,
        obj: &SchemaObject,
        name: &str,
        indent: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent_str = "    ".repeat(indent);
        output.push_str(&format!("{}export type {} = {{\n", indent_str, name));

        // Handle properties
        if let Some(properties) = &obj.properties {
            self.generate_properties(obj, properties, indent, output)?;
        }

        // Handle additionalProperties
        self.generate_additional_properties(obj, indent, output)?;

        output.push_str(&format!("{}}}", indent_str));
        Ok(())
    }

    fn generate_properties(
        &mut self,
        obj: &SchemaObject,
        properties: &HashMap<String, JsonSchema>,
        indent: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent_str = "    ".repeat(indent);
        let required_fields: HashSet<_> = obj
            .required
            .as_ref()
            .map(|r| r.iter().cloned().collect())
            .unwrap_or_default();

        let mut prop_names: Vec<_> = properties.keys().cloned().collect();
        prop_names.sort();

        for prop_name in prop_names {
            if let Some(prop_schema) = properties.get(&prop_name) {
                self.generate_property(
                    prop_schema,
                    &prop_name,
                    required_fields.contains(&prop_name),
                    &indent_str,
                    output,
                )?;
            }
        }

        Ok(())
    }

    fn generate_property(
        &mut self,
        prop_schema: &JsonSchema,
        prop_name: &str,
        is_required: bool,
        indent_str: &str,
        output: &mut String,
    ) -> Result<()> {
        // Add property description
        if let JsonSchema::Object(prop_obj) = prop_schema {
            if let Some(desc) = &prop_obj.description {
                output.push_str(&format!("{}    --- {}\n", indent_str, desc));
            }
        }

        let prop_type = self.inline_type(prop_schema)?;
        let constraints =
            self.format_constraints_with_indent(prop_schema, &format!("{}    ", indent_str));
        if !constraints.is_empty() {
            output.push_str(&constraints);
        }

        let optional_marker = if is_required { "" } else { "?" };
        output.push_str(&format!(
            "{}    {}: {}{},\n",
            indent_str, prop_name, prop_type, optional_marker
        ));

        Ok(())
    }

    fn generate_additional_properties(
        &mut self,
        obj: &SchemaObject,
        indent: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent_str = "    ".repeat(indent);
        if let Some(additional) = &obj.additional_properties {
            let add_type = match additional {
                AdditionalProperties::Boolean(true) => "any".to_string(),
                AdditionalProperties::Boolean(false) => return Ok(()), // No additional properties allowed
                AdditionalProperties::Schema(schema) => self.inline_type(schema)?,
            };
            output.push_str(&format!("{}    [string]: {},\n", indent_str, add_type));
        }
        Ok(())
    }

    fn generate_array_type(
        &mut self,
        obj: &SchemaObject,
        name: &str,
        indent: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent_str = "    ".repeat(indent);
        let item_type = if let Some(items) = &obj.items {
            self.inline_type(items)?
        } else {
            "any".to_string()
        };

        let constraints =
            self.format_constraints_with_indent(&JsonSchema::Object(obj.clone()), &indent_str);
        if !constraints.is_empty() {
            output.push_str(&constraints);
        }

        output.push_str(&format!(
            "{}export type {} = {{ {} }}",
            indent_str, name, item_type
        ));
        Ok(())
    }

    fn inline_type(&mut self, schema: &JsonSchema) -> Result<String> {
        match schema {
            JsonSchema::Boolean(true) => Ok("any".to_string()),
            JsonSchema::Boolean(false) => Ok("never".to_string()),
            JsonSchema::Object(obj) => self.inline_object_type(obj),
        }
    }

    fn inline_object_type(&mut self, obj: &SchemaObject) -> Result<String> {
        // Handle $ref
        if let Some(ref_path) = &obj.ref_ {
            return self.resolve_ref(ref_path);
        }

        // Handle enum and const
        if let Some(enum_values) = &obj.enum_ {
            return Ok(self.convert_enum(enum_values));
        }
        if let Some(const_value) = &obj.const_ {
            return Ok(self.convert_const(const_value));
        }

        // Handle composition types
        if let Some(result) = self.inline_composition_types(obj)? {
            return Ok(result);
        }

        // Handle type-specific inline conversion
        self.inline_type_specific(obj)
    }

    fn inline_composition_types(&mut self, obj: &SchemaObject) -> Result<Option<String>> {
        if let Some(any_of) = &obj.any_of {
            let types: Result<Vec<_>> = any_of.iter().map(|s| self.inline_type(s)).collect();
            return Ok(Some(format!("({})", types?.join(" | "))));
        }
        if let Some(one_of) = &obj.one_of {
            let types: Result<Vec<_>> = one_of.iter().map(|s| self.inline_type(s)).collect();
            return Ok(Some(format!("({})", types?.join(" | "))));
        }
        if let Some(all_of) = &obj.all_of {
            let types: Result<Vec<_>> = all_of.iter().map(|s| self.inline_type(s)).collect();
            return Ok(Some(format!("({})", types?.join(" & "))));
        }
        Ok(None)
    }

    fn inline_type_specific(&mut self, obj: &SchemaObject) -> Result<String> {
        if let Some(type_) = &obj.type_ {
            let types = Self::get_single_types(type_);

            // Handle union types
            if types.len() > 1 {
                let type_strings = self.map_types_to_strings(&types);
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
                    let item_type = if let Some(items) = &obj.items {
                        self.inline_type(items)?
                    } else {
                        "any".to_string()
                    };
                    Ok(format!("{{ {} }}", item_type))
                }
                SingleType::Object => self.inline_object_properties(obj),
            }
        } else if obj.properties.is_some() {
            Ok("{ [string]: any }".to_string())
        } else {
            Ok("any".to_string())
        }
    }

    fn inline_object_properties(&mut self, obj: &SchemaObject) -> Result<String> {
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
                    inline.push_str(&format!("{}: {}{}", prop_name, prop_type, optional_marker));
                }
            }
            inline.push_str(" }");
            Ok(inline)
        } else if let Some(additional) = &obj.additional_properties {
            let add_type = match additional {
                AdditionalProperties::Boolean(true) => "any".to_string(),
                AdditionalProperties::Boolean(false) => return Ok("{ }".to_string()),
                AdditionalProperties::Schema(schema) => self.inline_type(schema)?,
            };
            Ok(format!("{{ [string]: {} }}", add_type))
        } else {
            Ok("{ [string]: any }".to_string())
        }
    }

    fn convert_enum(&self, values: &[serde_json::Value]) -> String {
        let (all_strings, all_numbers) =
            values
                .iter()
                .fold((true, true), |(strings, numbers), v| match v {
                    serde_json::Value::String(_) => (strings, false),
                    serde_json::Value::Number(_) => (false, numbers),
                    serde_json::Value::Bool(_) | serde_json::Value::Null => (false, false),
                    _ => (false, false),
                });

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
            return Ok(def_name.to_case(Case::Pascal));
        }
        if let Some(def_name) = ref_path.strip_prefix("#/$defs/") {
            return Ok(def_name.to_case(Case::Pascal));
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

        let (separator, comment) = match kind {
            "allOf" => (" & ", "Intersection type (all conditions must be met)"),
            "anyOf" => (" | ", "Union type (any of these types)"),
            "oneOf" => (" | ", "Union type (exactly one of these types)"),
            _ => (" | ", "Combined type"),
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

    fn format_constraints_with_indent(&self, schema: &JsonSchema, indent_str: &str) -> String {
        let mut output = String::new();

        if let JsonSchema::Object(obj) = schema {
            self.add_numeric_constraints_indent(obj, indent_str, &mut output);
            self.add_string_constraints_indent(obj, indent_str, &mut output);
            self.add_array_constraints_indent(obj, indent_str, &mut output);
            self.add_object_constraints_indent(obj, indent_str, &mut output);
        }

        output
    }

    fn add_numeric_constraints_indent(
        &self,
        obj: &SchemaObject,
        indent_str: &str,
        output: &mut String,
    ) {
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
    }

    fn add_string_constraints_indent(
        &self,
        obj: &SchemaObject,
        indent_str: &str,
        output: &mut String,
    ) {
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
    }

    fn add_array_constraints_indent(
        &self,
        obj: &SchemaObject,
        indent_str: &str,
        output: &mut String,
    ) {
        if let Some(min_items) = obj.min_items {
            output.push_str(&format!("{}--- @minItems {}\n", indent_str, min_items));
        }
        if let Some(max_items) = obj.max_items {
            output.push_str(&format!("{}--- @maxItems {}\n", indent_str, max_items));
        }
        if let Some(true) = obj.unique_items {
            output.push_str(&format!("{}--- @uniqueItems true\n", indent_str));
        }
    }

    fn add_object_constraints_indent(
        &self,
        obj: &SchemaObject,
        indent_str: &str,
        output: &mut String,
    ) {
        if let Some(min_props) = obj.min_properties {
            output.push_str(&format!("{}--- @minProperties {}\n", indent_str, min_props));
        }
        if let Some(max_props) = obj.max_properties {
            output.push_str(&format!("{}--- @maxProperties {}\n", indent_str, max_props));
        }
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
