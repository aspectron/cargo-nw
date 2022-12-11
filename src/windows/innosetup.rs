use std::collections::HashMap;
use std::{rc::Rc, cell::RefCell};
use std::fmt;
use convert_case::{Case, Casing};


pub struct InnoSetup {
    definitions: Rc<Definitions>,
    sections: Vec<Rc<Section>>,
    map: HashMap<String, Rc<Section>>,
}

impl InnoSetup {
    pub fn new() -> InnoSetup {
        InnoSetup { 
            sections: Vec::new(),
            map: HashMap::new(),
            definitions: Rc::new(Definitions::new()),
        }
    }

    pub fn define(&self, k: &str, v: &str) -> Rc<Definitions> {
        self.definitions.clone().define(k, v);
        self.definitions.clone()
    }

    pub fn section(&mut self, name: &str) -> Rc<Section> {
        let section = Rc::new(Section::new(name));
        self.sections.push(section.clone());
        self.map.insert(name.to_lowercase(),section.clone());
        section
    }

    pub fn setup(&mut self) -> Rc<DirectivesSection> {
        self.section("Setup").as_directives()
    }

    pub fn icons(&mut self) -> Rc<Icons> {
        let section = self.section("Icons").as_args();
        Rc::new(Icons { section })
    }

    pub fn registry(&mut self) -> Rc<Registry> {
        let section = self.section("Registry").as_args();
        Rc::new(Registry { section })
    }

    pub fn tasks(&mut self) -> Rc<Tasks> {
        let section = self.section("Tasks").as_args();
        Rc::new(Tasks { section })
    }

    pub fn files(&mut self) -> Rc<FilesSection> {
        let section = self.section("Files").as_args();
        Rc::new(FilesSection { section })
    }

    pub fn run(&mut self) -> Rc<Run> {
        if let Some(section) = self.map.get("run") {
            Rc::new(Run { section : section.clone().as_args() })
        } else {
            let section = self.section("Run").as_args();
            Rc::new(Run { section })
        }
    }

    pub fn firewall(&mut self) -> Rc<Firewall> {
        if let Some(section) = self.map.get("run") {
            Rc::new(Firewall { section : section.clone().as_args() })
        } else {
            let section = self.section("Run").as_args();
            Rc::new(Firewall { section })
        }
    }

    pub fn install_delete(&mut self) -> Rc<ArgsSection> {
        let section = self.section("InstallDelete").as_args();
        section
    }


    pub fn languages(&mut self, languages: &[&str]) {
        let args = self.section("Languages").as_args();

        for lang in languages {
            let lang = lang.to_lowercase();
            let lang_title = lang.to_string()
            .from_case(Case::Lower)
            .to_case(Case::Title);
            if lang.as_str() == "english" {
                args.push(&[qs!("Name",lang_title),qs!("MessagesFile","compiler:Default.isl")])
            } else {
                args.push(&[qs!("Name",lang_title),qs!("MessagesFile",format!("compiler:Languages\\{lang_title}.isl"))])

            };
        }

    }


}

pub struct Definitions {
    args: Rc<RefCell<Vec<(String,String)>>>
}

impl Definitions {
    pub fn new() -> Definitions {
        Definitions {
            args: Rc::new(RefCell::new(Vec::new()))
        }
    }

    pub fn define(self : Rc<Self>, k: &str, v: &str) -> Rc<Self> {
        self.args.borrow_mut().push((k.to_string(),v.to_string()));
        self
    }

}

impl fmt::Display for Definitions {
    fn fmt(&self, f : &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        for (k,v) in self.args.borrow().iter() {
            writeln!(f, "#define {} \"{}\"",k,v)?;
        }

        Ok(())
    }
}

pub struct DirectivesSection {
    args: Rc<RefCell<Vec<(String,String)>>>
}

impl DirectivesSection {
    pub fn new() -> DirectivesSection {
        DirectivesSection {
            args: Rc::new(RefCell::new(Vec::new()))
        }
    }

    pub fn directives(self : Rc<Self>, list : &[(&str,&str)]) -> Rc<Self> {
        for (k,v) in list {
            self.args.borrow_mut().push((k.to_string(),v.to_string()));
        }

        self
    }
}

impl fmt::Display for DirectivesSection {
    fn fmt(&self, f : &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        for (k,v) in self.args.borrow().iter() {
            writeln!(f, "{} = {}",k,v)?;
        }

        Ok(())
    }
}

pub struct ArgsSection {
    args: Rc<RefCell<Vec<Vec<(String,String)>>>>
}

impl ArgsSection {
    pub fn new() -> ArgsSection {
        ArgsSection {
            args: Rc::new(RefCell::new(Vec::new()))
        }
    }

    pub fn args(self : Rc<Self>, list : &[&[(&str,&str)]]) -> Rc<Self> {
        for line in list {
            let mut args = Vec::new();
            for (k,v) in line.iter() {
                args.push((k.to_string(),v.to_string()));
            }
            self.args.borrow_mut().push(args);
        }

        self
    }

    pub fn push(self: &Rc<Self>, args: &[(String,String)]) {
        self.args.borrow_mut().push(args.to_vec());
    }
}

impl fmt::Display for ArgsSection {
    fn fmt(&self, f : &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        for entries in self.args.borrow().iter() {
            for (k,v) in entries.iter() {
                write!(f, "{}: {}; ",k,v)?;
            }
            writeln!(f, "")?;
        }

        Ok(())
    }
}

pub struct Icons {
    section : Rc<ArgsSection>
}

impl Icons {
    pub fn icon(self: Rc<Self>, name: &str, filename: &str, task : Option<&str>) -> Rc<Self> {

        let mut args = vec![
            qs!("Name", name),
            qs!("Filename", filename)
        ];
        if let Some(task) = task {
            args.push(("Task".into(), task.into()));
        }
        self.section.push(&args);
        self
    }
}


pub struct Registry {
    section : Rc<ArgsSection>
}

impl Registry {
    pub fn register(
        self: Rc<Self>,
        root: &str,
        subkey: &str,
        value_type: &str,
        value_name: &str,
        value_data: &str,
        flags : Option<&str>
    ) -> Rc<Self> {

        let mut args = vec![
            ("Root".to_string(), root.to_string()),
            qs!("Subkey", subkey),
            ("ValueType".to_string(), value_type.to_string()),
            qs!("ValueName", value_name),
            qs!("ValueData", value_data),
        ];
        if let Some(flags) = flags {
            args.push(("Flags".into(), flags.into()));
        }
        self.section.clone().push(&args);
        self
    }
}


pub struct Tasks {
    section : Rc<ArgsSection>
}

impl Tasks {
    pub fn task(
        self: Rc<Self>,
        name: &str,
        description: &str,
        group: &str,
        flags : Option<&str>,
        other : Option<&[(&str,&str)]>
    ) -> Rc<Self> {

        let mut args = vec![
            qs!("Name", name),
            qs!("Description", description),
            qs!("GroiupDescription", group),
        ];
        if let Some(flags) = flags {
            args.push(("Flags".into(), flags.into()));
        }
        if let Some(other) = other {
            for (k,v) in other.iter() {
                args.push((k.to_string(), v.to_string()));

            }
        }
        self.section.clone().push(&args);
        self
    }
}

pub struct FilesSection {
    section : Rc<ArgsSection>
}

impl FilesSection {
    pub fn replicate(
        self: Rc<Self>,
        source: &str,
        dest_dir: &str,
        flags : Option<&str>,
    ) -> Rc<Self> {

        let mut args = vec![
            qs!("Source",source),
            qs!("DestDir",dest_dir),
        ];
        if let Some(flags) = flags {
            args.push(("Flags".into(), flags.into()));
        }
        self.section.clone().push(&args);
        self
    }
}

pub struct Run {
    section : Rc<ArgsSection>
}

impl Run {
    pub fn exec(
        self: Rc<Self>,
        filename: &str,
        parameters: Option<&str>,
        description: Option<&str>,
        flags : Option<&str>,
    ) -> Rc<Self> {

        let mut args = vec![
            qs!("Filename",filename),
        ];
        if let Some(parameters) = parameters {
            args.push(qs!("Parameters",parameters));
        }
        if let Some(description) = description {
            args.push(qs!("Description",description));
        }
        if let Some(flags) = flags {
            args.push(("Flags".into(), flags.into()));
        }
        self.section.clone().push(&args);
        self
    }
}


pub struct Firewall {
    section : Rc<ArgsSection>
}

impl Firewall {

    pub fn add_rule(
        self: Rc<Self>,
        name: &str,
        file: &str,
        direction: &str,
    ) -> Rc<Self> {

        if direction.contains("in") {
            let args = vec![
                qs!("Filename","{sys}\\netsh.exe"),
                qs!("Parameters",format!("advfirewall firewall add rule name=\"\"{name}\"\" program=\"\"{{app}}\\{file}\"\" dir=in action=allow enable=yes")),
                ("Flags".to_string(),"runhidden".to_string()),
            ];
            self.section.push(&args);
        }

        if direction.contains("out") {
            let args = vec![
                qs!("Filename","{sys}\\netsh.exe"),
                qs!("Parameters",format!("advfirewall firewall add rule name=\"\"{name}\"\" program=\"\"{{app}}\\{file}\"\" dir=out action=allow enable=yes")),
                ("Flags".to_string(),"runhidden".to_string()),
            ];
            self.section.push(&args);
        }
        self
    }
}

pub enum SectionArgs {
    Directive(Rc<DirectivesSection>),
    MultiArg(Rc<ArgsSection>),
}

pub struct Section {
    name : String,
    args: Rc<RefCell<Option<SectionArgs>>>
}

impl Section {
    pub fn new(name: &str) -> Section {
        Section {
            name: name.to_string(),
            args: Rc::new(RefCell::new(None)),
        }
    }

    pub fn as_directives(self: Rc<Self>) -> Rc<DirectivesSection> {
        let mut section = self.args.borrow_mut();
        if let Some(section) = section.as_ref() {
            match section {
                SectionArgs::Directive(section) => section.clone(),
                SectionArgs::MultiArg(_) => panic!("InnoSetup - requesting incompatible section types"),
            }
        } else {
            let directives = Rc::new(DirectivesSection::new());
            section.replace(SectionArgs::Directive(directives.clone()));
            directives
        }
    }

    pub fn as_args(self: Rc<Self>) -> Rc<ArgsSection> {
        let mut section = self.args.borrow_mut();
        if let Some(section) = section.as_ref() {
            match section {
                SectionArgs::Directive(_) => panic!("InnoSetup - requesting incompatible section types"),
                SectionArgs::MultiArg(section) => section.clone(),
            }
        } else {
            let args = Rc::new(ArgsSection::new());
            section.replace(SectionArgs::MultiArg(args.clone()));
            args
        }
    }

}

impl fmt::Display for InnoSetup {
    fn fmt(&self, f : &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        writeln!(f, "{}", self.definitions.to_string())?;
        for section in &self.sections {
            writeln!(f, "[{}]", section.name)?;
            if let Some(section) = section.args.borrow().as_ref() {
                match section {
                    SectionArgs::Directive(section) => {
                        writeln!(f, "{}",section.to_string())?;
                    },
                    SectionArgs::MultiArg(section) => {
                        writeln!(f, "{}",section.to_string())?;
                    }
                }
            } else {
                panic!("InnoSetup: empty section '{}'", section.name)
            }
        }
        Ok(())
    }
}

#[macro_export]
macro_rules! quote {
    ($v:expr) => (
        &format!("\"{}\"",$v)
    )
}

pub use quote;

#[macro_export]
macro_rules! qs {
    ($k:expr, $v:expr) => (
        (String::from($k), format!("\"{}\"",$v))
    )
}

pub use qs;
