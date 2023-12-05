use cmd_lib::run_fun;

pub struct AppCmd {}

impl AppCmd {
    pub fn has_package(package_name: &str) -> Result<bool, String> {
        let res = run_fun!(which $package_name).map_err(|e| e.to_string())?;
        let contains_in_result = res.contains(package_name);
        Ok(contains_in_result)
    }
}
