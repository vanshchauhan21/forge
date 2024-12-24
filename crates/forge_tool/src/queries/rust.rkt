(struct_item
    name: (type_identifier) @name.definition.class) @definition.class

(function_item
    name: (identifier) @name.definition.function) @definition.function

(impl_item
    body: (declaration_list
        (function_item
            name: (identifier)
            parameters: (parameters)
            return_type: (_)?
            body: (_)?) @name.definition.method)) @definition.method
