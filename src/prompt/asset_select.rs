use inquire::{list_option::ListOption, validator::Validation, MultiSelect};

use crate::constants::NymReleaseAssets;

pub struct NymUpdateAssetSelectPrompt {
    options: Vec<AppSelectOption>,
}

impl NymUpdateAssetSelectPrompt {
    pub fn new(selected_options: Option<Vec<AppSelectOption>>) -> Self {
        let selected_names: Vec<String> = selected_options
            .unwrap_or_default()
            .iter()
            .filter(|option| option.checked)
            .map(|option| option.name.clone())
            .collect();

        let options = NymReleaseAssets::get_all_as_string()
            .iter()
            .enumerate()
            .map(|(index, name)| AppSelectOption {
                name: name.to_string(),
                checked: selected_names.contains(name),
                index,
            })
            .collect();

        Self { options }
    }

    fn get_list_options(&self) -> Vec<String> {
        self.options
            .iter()
            .map(|option| option.name.clone())
            .collect()
    }

    fn get_selected_options(&self) -> Vec<usize> {
        self.options
            .iter()
            .filter(|option| option.checked)
            .map(|option| option.index)
            .collect()
    }

    pub fn start(&self) -> Option<Vec<AppSelectOption>> {
        let validate_options = |a: &[ListOption<&String>]| {
            if a.len() > 0 {
                return Ok(Validation::Valid);
            }

            Ok(Validation::Invalid(
                "Remember to select at least one option".into(),
            ))
        };

        let promt_answers = MultiSelect::new(
            "Which application do you want to update ?, !!! Please be aware that selection can be made with spacebar!!! Selections ->",
            self.get_list_options(),
        )
        .with_default(&self.get_selected_options())
        .with_validator(validate_options)
        .prompt();

        match promt_answers {
            Ok(answers) => {
                let mut result: Vec<AppSelectOption> = Vec::new();
                for option in self.options.iter() {
                    let final_option = AppSelectOption {
                        name: option.name.clone(),
                        checked: answers.contains(&option.name),
                        index: option.index,
                    };
                    result.push(final_option);
                }
                println!("List of selected options: {:?}", result);

                return Some(result);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                return None;
            }
        }
    }
}

#[derive(Debug)]
pub struct AppSelectOption {
    pub name: String,
    pub checked: bool,
    pub index: usize,
}
