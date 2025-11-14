use convert_case::{Case, Casing};
use std::collections::{HashMap, HashSet};

use crate::error::{ConversionError, Result};
use crate::schema::{AdditionalProperties, JsonSchema, SchemaObject, SchemaType, SingleType};

/// Converts JSON Schema to Luau type definitions
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

    /// Convert schema to Luau type definitions with default root name
    pub fn convert(&self, schema: &JsonSchema) -> Result<String> {
        self.convert_with_name(schema, "Root")
    }

    /// Convert schema to Luau type definitions with custom type name
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

    /// Extract definitions from schema object
    fn extract_definitions(&mut self, schema: &JsonSchema) {
        if let JsonSchema::Object(obj) = schema {
            // Extract from both definitions and $defs
            for defs in [&obj.definitions, &obj.defs].into_iter().flatten() {
                self.definitions.extend(defs.clone());
            }
        }
    }

    /// Generate all definition types in sorted order
    fn generate_definitions(&mut self, output: &mut String) -> Result<()> {
        let mut def_names: Vec<_> = self.definitions.keys().cloned().collect();
        def_names.sort();

        for def_name in def_names {
            let pascal_def_name = def_name.to_case(Case::Pascal);
            if !self.generated_types.contains(&pascal_def_name)
                && let Some(def_schema) = self.definitions.get(&def_name).cloned()
            {
                output.push_str("\n\n");
                let def_type = self.convert_schema(&def_schema, &pascal_def_name, 0)?;
                output.push_str(&def_type);
            }
        }

        Ok(())
    }

    /// Main schema conversion entry point
    fn convert_schema(&mut self, schema: &JsonSchema, name: &str, indent: usize) -> Result<String> {
        match schema {
            JsonSchema::Boolean(true) => Ok("any".to_string()),
            JsonSchema::Boolean(false) => Ok("never".to_string()),
            JsonSchema::Object(obj) => self.convert_object(obj, name, indent),
        }
    }

    /// Extract single types from SchemaType
    fn get_single_types(schema_type: &SchemaType) -> Vec<&SingleType> {
        match schema_type {
            SchemaType::Single(single) => vec![single],
            SchemaType::Multiple(types) => types.iter().collect(),
        }
    }

    /// Convert schema object to type definition
    fn convert_object(&mut self, obj: &SchemaObject, name: &str, indent: usize) -> Result<String> {
        let indent_str = Self::create_indent(indent);

        // Add description as comment if present
        let description_comment = if let Some(desc) = &obj.description {
            format!("{}--- {}\n", indent_str, desc)
        } else {
            String::new()
        };

        // Handle references
        if let Some(ref_path) = &obj.ref_ {
            return self.resolve_ref(ref_path).map(|resolved| {
                format!(
                    "{}{}export type {} = {}",
                    description_comment, indent_str, name, resolved
                )
            });
        }

        // Handle composition types (allOf, anyOf, oneOf)
        if let Some(mut result) = self.handle_composition_types(obj, name, indent)? {
            if !description_comment.is_empty() {
                result = format!("{}{}", description_comment, result);
            }
            return Ok(result);
        }

        // Handle enum and const values
        if let Some(enum_values) = &obj.enum_ {
            let mut result = self.generate_enum_type(enum_values, name, &indent_str)?;
            if !description_comment.is_empty() {
                result = format!("{}{}", description_comment, result);
            }
            return Ok(result);
        }

        if let Some(const_value) = &obj.const_ {
            let mut result = self.generate_const_type(const_value, name, &indent_str)?;
            if !description_comment.is_empty() {
                result = format!("{}{}", description_comment, result);
            }
            return Ok(result);
        }

        // Handle type-specific conversion
        let mut result = self.handle_type_conversion(obj, name, indent)?;
        if !description_comment.is_empty() {
            result = format!("{}{}", description_comment, result);
        }
        Ok(result)
    }

    /// Handle composition types: allOf, anyOf, oneOf
    fn handle_composition_types(
        &mut self,
        obj: &SchemaObject,
        name: &str,
        indent: usize,
    ) -> Result<Option<String>> {
        match self.get_composition_type(obj) {
            Some(("allOf", schemas)) => self.handle_all_of(obj, schemas, name, indent),
            Some(("anyOf", schemas)) => self.handle_union_type(
                schemas,
                name,
                indent,
                "anyOf",
                "Union type (any of these types)",
            ),
            Some(("oneOf", schemas)) => self.handle_union_type(
                schemas,
                name,
                indent,
                "oneOf",
                "Union type (exactly one of these types)",
            ),
            _ => Ok(None),
        }
    }

    /// Get composition type and schemas if present
    fn get_composition_type<'a>(
        &self,
        obj: &'a SchemaObject,
    ) -> Option<(&'static str, &'a Vec<JsonSchema>)> {
        if let Some(all_of) = &obj.all_of {
            Some(("allOf", all_of))
        } else if let Some(any_of) = &obj.any_of {
            Some(("anyOf", any_of))
        } else if let Some(one_of) = &obj.one_of {
            Some(("oneOf", one_of))
        } else {
            None
        }
    }

    /// Handle allOf composition with merging or intersection
    fn handle_all_of(
        &mut self,
        obj: &SchemaObject,
        all_of: &[JsonSchema],
        name: &str,
        indent: usize,
    ) -> Result<Option<String>> {
        let parent_has_props = obj.properties.is_some()
            || obj.additional_properties.is_some()
            || obj.required.is_some();

        if parent_has_props {
            // Merge parent properties with allOf schemas
            let merged = self.merge_all_of_schemas(obj, all_of)?;
            self.convert_object(&merged, name, indent).map(Some)
        } else {
            // Create intersection type
            self.handle_union_type(
                all_of,
                name,
                indent,
                "allOf",
                "Intersection type (all conditions must be met)",
            )
        }
    }

    /// Merge allOf schemas into parent schema
    fn merge_all_of_schemas(
        &mut self,
        parent: &SchemaObject,
        all_of: &[JsonSchema],
    ) -> Result<SchemaObject> {
        let mut merged = parent.clone();
        merged.all_of = None;
        // Remove description to prevent duplicate comments when recursively converting
        merged.description = None;

        for sub in all_of {
            if let JsonSchema::Object(sub_obj) = sub {
                let resolved_obj = self.resolve_reference_if_needed(sub_obj);

                // Merge properties
                if let Some(sub_props) = &resolved_obj.properties {
                    merged
                        .properties
                        .get_or_insert_with(Default::default)
                        .extend(sub_props.clone());
                }

                // Merge required fields
                if let Some(sub_req) = &resolved_obj.required {
                    merged
                        .required
                        .get_or_insert_with(Default::default)
                        .extend(sub_req.clone());
                }

                // Merge additionalProperties (last one wins)
                if let Some(additional) = &resolved_obj.additional_properties {
                    merged.additional_properties = Some(additional.clone());
                }
            }
        }

        Ok(merged)
    }

    /// Resolve reference if object is a $ref
    fn resolve_reference_if_needed<'a>(&'a self, obj: &'a SchemaObject) -> &'a SchemaObject {
        if let Some(ref_path) = &obj.ref_
            && let Some(def_name) = ref_path
                .strip_prefix("#/$defs/")
                .or_else(|| ref_path.strip_prefix("#/definitions/"))
            && let Some(JsonSchema::Object(ref_obj)) = self.definitions.get(def_name)
        {
            return ref_obj;
        }
        obj
    }

    /// Handle union types (anyOf, oneOf) and intersection (allOf without parent props)
    fn handle_union_type(
        &mut self,
        schemas: &[JsonSchema],
        name: &str,
        indent: usize,
        kind: &str,
        comment: &str,
    ) -> Result<Option<String>> {
        let indent_str = Self::create_indent(indent);
        let types: Result<Vec<_>> = schemas.iter().map(|s| self.inline_type(s)).collect();

        let separator = if kind == "allOf" { " & " } else { " | " };

        self.generated_types.insert(name.to_string());

        Ok(Some(format!(
            "{}--- {}\n{}export type {} = {}",
            indent_str,
            comment,
            indent_str,
            name,
            types?.join(separator)
        )))
    }

    /// Handle type-specific conversion logic
    fn handle_type_conversion(
        &mut self,
        obj: &SchemaObject,
        name: &str,
        indent: usize,
    ) -> Result<String> {
        let indent_str = Self::create_indent(indent);
        self.generated_types.insert(name.to_string());

        if let Some(type_) = &obj.type_ {
            let types = Self::get_single_types(type_);

            // Handle union types (multiple types)
            if types.len() > 1 {
                return self.generate_union_type(obj, name, &types, &indent_str);
            }

            // Handle single type
            let single_type = types[0];
            self.generate_single_type(obj, name, single_type, indent)
        } else if obj.properties.is_some() || obj.additional_properties.is_some() {
            // Infer as object if properties exist
            self.generate_object_type(obj, name, indent)
        } else {
            Ok(format!("{}export type {} = any", indent_str, name))
        }
    }

    /// Generate union type for multiple possible types
    fn generate_union_type(
        &mut self,
        obj: &SchemaObject,
        name: &str,
        types: &[&SingleType],
        indent_str: &str,
    ) -> Result<String> {
        let type_strings = self.map_types_to_strings(types);
        let constraints = self
            .format_constraints_with_indent(&JsonSchema::Object(Box::new(obj.clone())), indent_str);

        Ok(format!(
            "{}{}export type {} = {}",
            constraints,
            indent_str,
            name,
            type_strings.join(" | ")
        ))
    }

    /// Generate type for a single schema type
    fn generate_single_type(
        &mut self,
        obj: &SchemaObject,
        name: &str,
        single_type: &SingleType,
        indent: usize,
    ) -> Result<String> {
        let indent_str = Self::create_indent(indent);

        match single_type {
            SingleType::Object => self.generate_object_type(obj, name, indent),
            SingleType::Array => self.generate_array_type(obj, name, indent),
            SingleType::String | SingleType::Number | SingleType::Integer => {
                let type_name = match single_type {
                    SingleType::String => "string",
                    SingleType::Number | SingleType::Integer => "number",
                    _ => unreachable!(),
                };
                let constraints = self.format_constraints_with_indent(
                    &JsonSchema::Object(Box::new(obj.clone())),
                    &indent_str,
                );

                Ok(format!(
                    "{}{}export type {} = {}",
                    constraints, indent_str, name, type_name
                ))
            }
            SingleType::Boolean => Ok(format!("{}export type {} = boolean", indent_str, name)),
            SingleType::Null => Ok(format!("{}export type {} = nil", indent_str, name)),
        }
    }

    /// Map SingleType variants to their string representations
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

    /// Generate object type definition
    fn generate_object_type(
        &mut self,
        obj: &SchemaObject,
        name: &str,
        indent: usize,
    ) -> Result<String> {
        let indent_str = Self::create_indent(indent);
        let mut output = format!("{}export type {} = {{\n", indent_str, name);

        // Handle properties
        if let Some(properties) = &obj.properties {
            self.generate_properties(obj, properties, indent, &mut output)?;
        }

        // Handle additionalProperties
        self.generate_additional_properties(obj, indent, &mut output)?;

        output.push_str(&format!("{}}}", indent_str));
        Ok(output)
    }

    /// Generate properties for object type
    fn generate_properties(
        &mut self,
        obj: &SchemaObject,
        properties: &HashMap<String, JsonSchema>,
        indent: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent_str = Self::create_indent(indent);
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

    /// Generate individual property
    fn generate_property(
        &mut self,
        prop_schema: &JsonSchema,
        prop_name: &str,
        is_required: bool,
        indent_str: &str,
        output: &mut String,
    ) -> Result<()> {
        // Add property description
        if let JsonSchema::Object(prop_obj) = prop_schema
            && let Some(desc) = &prop_obj.description
        {
            output.push_str(&format!("{}    --- {}\n", indent_str, desc));
        }

        let prop_type = self.inline_type(prop_schema)?;
        let constraints =
            self.format_constraints_with_indent(prop_schema, &format!("{}    ", indent_str));
        if !constraints.is_empty() {
            output.push_str(&constraints);
        }

        // Add format constraints for additionalProperties if they exist
        if let JsonSchema::Object(prop_obj) = prop_schema
            && let Some(AdditionalProperties::Schema(additional_schema)) =
                &prop_obj.additional_properties
        {
            let additional_constraints = self
                .format_constraints_with_indent(additional_schema, &format!("{}    ", indent_str));
            if !additional_constraints.is_empty() {
                output.push_str(&additional_constraints);
            }
        }

        let optional_marker = if is_required { "" } else { "?" };
        output.push_str(&format!(
            "{}    {}: {}{},\n",
            indent_str, prop_name, prop_type, optional_marker
        ));

        Ok(())
    }

    /// Generate additional properties definition
    fn generate_additional_properties(
        &mut self,
        obj: &SchemaObject,
        indent: usize,
        output: &mut String,
    ) -> Result<()> {
        let indent_str = Self::create_indent(indent);
        if let Some(additional) = &obj.additional_properties {
            let add_type = match additional {
                AdditionalProperties::Boolean(true) => "any".to_string(),
                AdditionalProperties::Boolean(false) => return Ok(()), // No additional properties allowed
                AdditionalProperties::Schema(schema) => {
                    // Add format constraints for additional properties if they exist
                    if let JsonSchema::Object(_) = schema.as_ref() {
                        let constraints = self
                            .format_constraints_with_indent(schema, &format!("{}    ", indent_str));
                        if !constraints.is_empty() {
                            output.push_str(&constraints);
                        }
                    }
                    self.inline_type(schema)?
                }
            };
            output.push_str(&format!("{}    [string]: {},\n", indent_str, add_type));
        }
        Ok(())
    }

    /// Generate array type definition
    fn generate_array_type(
        &mut self,
        obj: &SchemaObject,
        name: &str,
        indent: usize,
    ) -> Result<String> {
        let indent_str = Self::create_indent(indent);
        let item_type = if let Some(items) = &obj.items {
            self.inline_type(items)?
        } else {
            "any".to_string()
        };

        let constraints = self.format_constraints_with_indent(
            &JsonSchema::Object(Box::new(obj.clone())),
            &indent_str,
        );

        Ok(format!(
            "{}{}export type {} = {{ {} }}",
            constraints, indent_str, name, item_type
        ))
    }

    /// Generate enum type definition
    fn generate_enum_type(
        &mut self,
        values: &[serde_json::Value],
        name: &str,
        indent_str: &str,
    ) -> Result<String> {
        self.generated_types.insert(name.to_string());
        let union = self.convert_enum(values);
        Ok(format!("{}export type {} = {}", indent_str, name, union))
    }

    /// Generate const type definition
    fn generate_const_type(
        &mut self,
        value: &serde_json::Value,
        name: &str,
        indent_str: &str,
    ) -> Result<String> {
        self.generated_types.insert(name.to_string());
        let literal = self.convert_const(value);
        Ok(format!("{}export type {} = {}", indent_str, name, literal))
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
        // For brevity, keeping the original implementation here
        if let Some(any_of) = &obj.any_of {
            let types: Result<Vec<_>> = any_of.iter().map(|s| self.inline_type(s)).collect();
            return Ok(Some(format!("({})", types?.join(" | "))));
        }
        if let Some(one_of) = &obj.one_of {
            let types: Result<Vec<_>> = one_of.iter().map(|s| self.inline_type(s)).collect();
            return Ok(Some(format!("({})", types?.join(" | "))));
        }
        if let Some(all_of) = &obj.all_of {
            let parent_has_props = obj.properties.is_some()
                || obj.additional_properties.is_some()
                || obj.required.is_some();

            if parent_has_props {
                let mut merged_props = obj.properties.clone().unwrap_or_default();
                let mut merged_required = obj.required.clone().unwrap_or_default();
                let mut ref_types = Vec::new();

                for sub in all_of {
                    if let JsonSchema::Object(sub_obj) = sub {
                        if let Some(ref_path) = &sub_obj.ref_ {
                            ref_types.push(self.resolve_ref(ref_path)?);
                        } else {
                            if let Some(sub_props) = &sub_obj.properties {
                                merged_props.extend(sub_props.clone());
                            }
                            if let Some(sub_req) = &sub_obj.required {
                                merged_required.extend(sub_req.clone());
                            }
                        }
                    }
                }

                let mut merged_obj = obj.clone();
                merged_obj.properties = Some(merged_props);
                merged_obj.required = Some(merged_required);
                merged_obj.all_of = None;

                let merged_part = self.inline_object_properties(&merged_obj)?;

                if ref_types.is_empty() {
                    return Ok(Some(merged_part));
                } else {
                    let mut parts = vec![merged_part];
                    parts.extend(ref_types);
                    return Ok(Some(format!("({})", parts.join(" & "))));
                }
            }

            let types: Result<Vec<_>> = all_of.iter().map(|s| self.inline_type(s)).collect();
            return Ok(Some(format!("({})", types?.join(" & "))));
        }
        Ok(None)
    }

    fn inline_type_specific(&mut self, obj: &SchemaObject) -> Result<String> {
        if let Some(type_) = &obj.type_ {
            let types = Self::get_single_types(type_);

            if types.len() > 1 {
                let type_strings = self.map_types_to_strings(&types);
                return Ok(format!("({})", type_strings.join(" | ")));
            }

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

        if all_numbers {
            return "number".to_string();
        }

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

    /// Create indent string for given level
    fn create_indent(indent: usize) -> String {
        "    ".repeat(indent)
    }

    /// Constraint formatting methods remain the same
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

    /// Add numeric constraints with indentation
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

    /// Add string constraints with indentation
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

    /// Add array constraints with indentation
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

    /// Add object constraints with indentation
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
