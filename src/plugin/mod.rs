pub trait Plugin {
    fn name() -> String;
    fn is_supported() -> bool;
    fn handle() -> Result<(), ()>;
}
