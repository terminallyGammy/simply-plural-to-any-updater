#[macro_export]
macro_rules! record_if_error {
    ($self:expr, $result:expr) => {{
        let operation = $result;
        $self.last_operation_error = operation.as_ref().err().map(|e| e.to_string());
        operation
    }};
}
