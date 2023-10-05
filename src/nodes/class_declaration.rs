use crate::interpreter::ast_interpreter::{Execute, Interpreter};
use crate::nodes::AstExpression;
use crate::nodes::function_signature::FunctionSignature;
use crate::nodes::identifier::IdentifierNode;
use crate::value::function::JsFunction;
use crate::value::JsValue;
use crate::value::object::JsObject;

const CONSTRUCTOR_METHOD_NAME: &'static str = "constructor";

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclarationNode {
    pub name: Box<IdentifierNode>,
    pub parent: Option<Box<IdentifierNode>>,
    pub methods: Vec<Box<ClassMethodNode>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClassMethodNode {
    pub function_signature: FunctionSignature,
}

impl Execute for ClassDeclarationNode {
    fn execute(&self, interpreter: &Interpreter) -> Result<JsValue, String> {
        let prototype_object = self.build_prototype_object_from_class_declaration(interpreter);
        let mut constructor_function = self.build_constructor_from_class_declaration(interpreter).to_object();

        constructor_function.set_prototype(prototype_object.to_ref());

        let constructor_function = JsValue::Object(constructor_function.to_ref());

        interpreter.environment.borrow().borrow_mut().define_variable(
            self.name.id.clone(),
            constructor_function.clone(),
            false
        ).unwrap();

        Ok(constructor_function)
    }
}

impl ClassDeclarationNode {
    fn build_prototype_object_from_class_declaration(&self, interpreter: &Interpreter) -> JsObject {
        let mut prototype_object = JsObject::empty();

        for class_method in &self.methods {
            let method_value = interpreter.create_js_function(&class_method.function_signature.arguments, *class_method.function_signature.body.clone());

            prototype_object.add_property(&class_method.function_signature.name.id, method_value.into());
            // if let AstStatement::FunctionDeclaration(method_declaration) = &class_method {
            // if method_declaration.name.id == CONSTRUCTOR_METHOD_NAME { continue; }

            // let function = self.eval_function_declaration(&method_declaration).unwrap();
            //
            // if let IdentifierNode { id, .. } = method_declaration.function_signature.name.as_ref() {
            //     prototype_object.add_property(id.as_str(), function);
            // }
            // }
        }

        prototype_object
    }

    pub(crate) fn build_constructor_from_class_declaration(&self, interpreter: &Interpreter) -> JsFunction {
        let constructor_method = self.methods.iter().find(|x| {
            return x.function_signature.name.id == CONSTRUCTOR_METHOD_NAME;
        });

        if constructor_method.is_some() {
            let function_signature = &constructor_method.unwrap().as_ref().function_signature;
            interpreter.create_js_function(&function_signature.arguments, *function_signature.body.clone())
        } else {
            JsFunction::empty().into()
        }
    }
}

impl Into<AstExpression> for ClassDeclarationNode {
    fn into(self) -> AstExpression {
        AstExpression::ClassDeclaration(self)
    }
}

