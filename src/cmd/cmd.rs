use cmd_lib::{run_cmd, run_fun};
use tracing::info;

pub struct AppCmd {}

impl AppCmd {
    pub fn echo(msg: &str) -> Result<(), Box<dyn std::error::Error>> {
        run_cmd!(echo $msg)?;
        Ok(())
    }
    pub fn has_package(package_name: &str) -> Result<bool, Box<dyn std::error::Error>> {
        info!("checking if package exists -> '{}'", package_name);

        let res = run_fun!(which $package_name);
        if res.is_err() {
            return Ok(false);
        }

        let contains_in_result = res.unwrap().contains(package_name);
        if !contains_in_result {
            let not_found = format!("package not found -> '{}'", package_name);
            run_cmd!(echo $not_found)?;
            return Err(format!("package not found -> '{}'", package_name).into());
        }

        info!("package exists -> '{}'", package_name);
        Ok(contains_in_result)
    }
    pub fn install_if_not_exists(package_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let exists = AppCmd::has_package(package_name)?;
        if exists {
            return Ok(());
        }

        let installing = format!("installing package -> '{}'", package_name);
        run_cmd!(echo $installing)?;
        run_cmd!(sudo apt-get install $package_name -y -qq < "/dev/null")?;

        let installed = format!("package installed -> '{}'", package_name);
        run_cmd!(echo $installed)?;

        Ok(())
    }
}
