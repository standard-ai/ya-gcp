#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Expr {
    /// Textual representation of an expression in Common Expression Language
    /// syntax.
    #[prost(string, tag = "1")]
    pub expression: ::prost::alloc::string::String,
    /// Optional. Title for the expression, i.e. a short string describing
    /// its purpose. This can be used e.g. in UIs which allow to enter the
    /// expression.
    #[prost(string, tag = "2")]
    pub title: ::prost::alloc::string::String,
    /// Optional. Description of the expression. This is a longer text which
    /// describes the expression, e.g. when hovered over it in a UI.
    #[prost(string, tag = "3")]
    pub description: ::prost::alloc::string::String,
    /// Optional. String indicating the location of the expression for error
    /// reporting, e.g. a file name and a position in the file.
    #[prost(string, tag = "4")]
    pub location: ::prost::alloc::string::String,
}
