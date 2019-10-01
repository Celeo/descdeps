pub trait Driver {
    fn get_info(self, name: &str) -> String;
}
