use std::path::Path;

use cmd_lib::run_fun;
use tracing::info;

use crate::constants::NymReleaseAssets;

pub struct NymSystemdFileUtil {
    asset: NymReleaseAssets,
}

impl NymSystemdFileUtil {
    pub fn new(asset: NymReleaseAssets) -> Self {
        Self { asset }
    }

    fn get_service_path(&self) -> String {
        format!("/etc/systemd/system/{}.service", self.asset.name())
    }

    pub fn _has_asset_service(&self) -> bool {
        Path::new(&self.get_service_path()).exists()
    }

    pub async fn current_exec_start_path(&self) -> Result<String, String> {
        let asset_name = self.asset.name();
        let res = run_fun!(systemctl show -p ExecStart --value $asset_name | grep -o "path=[^;]*" | cut -d= -f2).map_err(|e| format!("Error while getting mixnode systemd path with {} error", e))?;
        info!("Mixnode systemd path is {}", res);

        Ok(res)
    }

    pub async fn exec_start_full_line(&self) -> Result<String, String> {
        let asset_name = self.asset.name();
        let line =
        run_fun!(systemctl show -p ExecStart --value $asset_name | grep -o r#"argv\[\]=[^;]*"# | cut -d= -f2)
            .map_err(|e| {
                format!("Error while getting {} systemd path with {} error", asset_name,e)
            })?;

        info!("{} systemd full line is {}", asset_name, line);

        Ok(line)
    }

    pub async fn update_exec_start_prop(&self, new_path: String) -> Result<(), String> {
        let asset_name = self.asset.name();
        let current_exec_start_path = self.current_exec_start_path().await?;
        let current_full_exec_start_line = self.exec_start_full_line().await?;

        let final_exec_start_line = current_full_exec_start_line.replace(
            &current_exec_start_path,
            //With one space line to separate the path and the args
            format!("{} ", new_path).as_str(),
        );

        let prop = NymSystemDProperty::ExecStart;
        self.set_service_property(&prop, &final_exec_start_line)?;

        info!(
            "{} systemd property {} updated to {}",
            asset_name,
            prop.as_str(),
            final_exec_start_line
        );

        Ok(())
    }

    pub async fn update_description_prop(&self, version: String) -> Result<(), String> {
        let asset_name = self.asset.name();
        let final_description = format!("Nym {} {}", asset_name, version);
        let prop = NymSystemDProperty::Description;
        self.set_service_property(&prop, &final_description)?;

        info!(
            "{} systemd {} property updated to {}",
            asset_name,
            prop.as_str(),
            final_description
        );

        Ok(())
    }

    pub fn systemd_reload(&self) -> Result<(), String> {
        run_fun!(sudo systemctl daemon-reload)
            .map_err(|e| format!("Error while reloading systemd with {} error", e))?;

        info!("Systemd reloaded");

        Ok(())
    }

    fn set_service_property(
        &self,
        property: &NymSystemDProperty,
        value: &str,
    ) -> Result<(), String> {
        let service_path = self.get_service_path();
        let prop_key = property.as_str();
        let property_full_line_str = format!("s|^{}=.*|{}={}|", prop_key, prop_key, value);

        run_fun!(sudo sed -i $property_full_line_str $service_path).map_err(|e| {
            format!(
                "Error while updating {} systemd file propery {} with {}. Error {}",
                service_path, prop_key, value, e
            )
        })?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum NymSystemDProperty {
    Description,
    ExecStart,
}

impl NymSystemDProperty {
    pub fn as_str(&self) -> &str {
        match self {
            NymSystemDProperty::Description => "Description",
            NymSystemDProperty::ExecStart => "ExecStart",
        }
    }
}
