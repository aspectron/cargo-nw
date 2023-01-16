use serde::Deserialize;

/// Application image and icon names (overrides)
#[derive(Default, Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Images {
    pub document: Option<String>,
    pub windows_document: Option<String>,
    pub linux_document: Option<String>,
    pub macos_document: Option<String>,
    pub application: Option<String>,
    pub windows_application: Option<String>,
    pub linux_application: Option<String>,
    pub macos_application: Option<String>,
    pub macos_disk_image: Option<String>,
    pub innosetup_icon: Option<String>,
    pub innosetup_wizard_small: Option<String>,
    pub innosetup_wizard_large: Option<String>,
}

fn list(user: &[Option<String>], defaults: &[&str]) -> Vec<String> {
    let mut list = user.to_vec();
    list.extend(
        defaults
            .iter()
            .map(|s| Some(s.to_string()))
            .collect::<Vec<_>>(),
    );
    list.iter().flatten().cloned().collect::<Vec<String>>()
}

impl Images {
    // CURRENTLY NOT USED
    pub fn windows_document(&self) -> Vec<String> {
        list(
            &[
                self.windows_document.clone(),
                self.document.clone(),
                self.application.clone(),
            ],
            &[
                "windows-document.png",
                "document.png",
                "default-application-icon.png",
            ],
        )
    }
    // MacOS document icon
    pub fn macos_document(&self) -> Vec<String> {
        list(
            &[
                self.macos_document.clone(),
                self.document.clone(),
                self.application.clone(),
            ],
            &[
                "macos-document.png",
                "document.png",
                "default-application-icon.png",
            ],
        )
    }
    // CURRENTLY NOT USED
    pub fn linux_document(&self) -> Vec<String> {
        list(
            &[
                self.linux_document.clone(),
                self.document.clone(),
                self.application.clone(),
            ],
            &[
                "linux-document.png",
                "document.png",
                "default-application-icon.png",
            ],
        )
    }
    // Windows application icon
    pub fn windows_application(&self) -> Vec<String> {
        list(
            &[self.windows_application.clone(), self.application.clone()],
            &[
                "windows-application.png",
                "application.png",
                "default-application-icon.png",
            ],
        )
    }
    /// MacOS application icon
    pub fn macos_application(&self) -> Vec<String> {
        list(
            &[self.macos_application.clone(), self.application.clone()],
            &[
                "macos-application.png",
                "application.png",
                "default-application-icon.png",
            ],
        )
    }
    /// MacOS Disk Image (DMG) window background
    pub fn macos_disk_image(&self) -> Vec<String> {
        list(
            &[self.macos_disk_image.clone()],
            &["macos-disk-image-background.png"],
        )
    }
    /// Linux application icon (CURRENTLY NOT USED)
    pub fn linux_application(&self) -> Vec<String> {
        list(
            &[self.linux_application.clone(), self.application.clone()],
            &[
                "linux-application.png",
                "application.png",
                "default-application-icon.png",
            ],
        )
    }
    /// InnoSetup installer icon
    pub fn innosetup_icon(&self) -> Vec<String> {
        list(
            &[self.innosetup_icon.clone(), self.application.clone()],
            &[
                "innosetup.png",
                "windows-application.png",
                "application.png",
                "default-application-icon.png",
            ],
        )
    }

    pub fn innosetup_wizard_small(&self) -> Vec<String> {
        list(
            &[self.innosetup_wizard_small.clone()],
            &["innosetup-wizard-small.png"],
        )
    }

    pub fn innosetup_wizard_large(&self) -> Vec<String> {
        list(
            &[self.innosetup_wizard_large.clone()],
            &["innosetup-wizard-large.png"],
        )
    }
}
