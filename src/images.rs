use serde::Deserialize;

/// Application image and icon names (overrides)
#[derive(Debug, Clone, Deserialize)]
pub struct Images {
    pub windows_document : Option<String>,
    pub linux_document : Option<String>,
    pub macos_document : Option<String>,
    pub windows_application : Option<String>,
    pub linux_application : Option<String>,
    pub macos_application : Option<String>,
    pub macos_disk_image : Option<String>,
    pub innosetup_icon : Option<String>,
}

impl Default for Images {
    fn default() -> Self {
        Self {
            // application: None,
            windows_document: None,
            linux_document: None,
            macos_document: None,
            windows_application: None,
            linux_application: None,
            macos_application: None,
            macos_disk_image: None,
            innosetup_icon: None,
        }
    }
}

impl Images {

    // CURRENTLY NOT USED
    pub fn windows_document(&self) -> Vec<String> {
        self.windows_document.as_ref().map(|s|vec![s.to_owned()]).unwrap_or(vec![
            "windows-document.png",
            "document.png",
            "default-application-icon.png",
        ].iter().map(|s|s.to_string()).collect::<Vec<_>>())
    }
    // MacOS document icon
    pub fn macos_document(&self) -> Vec<String> {
        self.macos_document.as_ref().map(|s|vec![s.to_owned()]).unwrap_or(vec![
            "macos-document.png",
            "document.png",
            "default-application-icon.png",
        ].iter().map(|s|s.to_string()).collect::<Vec<_>>())
    }
    // CURRENTLY NOT USED
    pub fn linux_document(&self) -> Vec<String> {
        self.linux_document.as_ref().map(|s|vec![s.to_owned()]).unwrap_or(vec![
            "linux-document.png",
            "document.png",
            "default-application-icon.png",
        ].iter().map(|s|s.to_string()).collect::<Vec<_>>())
    }
    // Windows application icon
    pub fn windows_application(&self) -> Vec<String> {
        self.windows_application.as_ref().map(|s|vec![s.to_owned()]).unwrap_or(vec![
            "windows-application.png",
            "application.png",
            "default-application-icon.png",
        ].iter().map(|s|s.to_string()).collect::<Vec<_>>())
    }
    /// MacOS application icon
    pub fn macos_application(&self) -> Vec<String> {
        self.macos_application.as_ref().map(|s|vec![s.to_owned()]).unwrap_or(vec![
            "macos-application.png",
            "application.png",
            "default-application-icon.png",
        ].iter().map(|s|s.to_string()).collect::<Vec<_>>())
    }
    /// MacOS Disk Image (DMG) window background
    pub fn macos_disk_image(&self) -> Vec<String> {
        self.macos_disk_image.as_ref().map(|s|vec![s.to_owned()]).unwrap_or(vec![
            "macos-dmg-background.png",
            "application.png",
            "default-application-icon.png",
        ].iter().map(|s|s.to_string()).collect::<Vec<_>>())
    }
    /// Linux application icon (CURRENTLY NOT USED)
    pub fn linux_application(&self) -> Vec<String> {
        self.linux_application.as_ref().map(|s|vec![s.to_owned()]).unwrap_or(vec![
            "linux-application.png",
            "application.png",
            "default-application-icon.png",
        ].iter().map(|s|s.to_string()).collect::<Vec<_>>())
    }
    /// InnoSetup installer icon
    pub fn innosetup_icon(&self) -> Vec<String> {
        self.innosetup_icon.as_ref().map(|s|vec![s.to_owned()]).unwrap_or(vec![
            "innosetup.png",
            "windows-application.png",
            "application.png",
            "default-application-icon.png",
        ].iter().map(|s|s.to_string()).collect::<Vec<_>>())
    }
}
