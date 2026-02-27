use std::collections::HashMap;

pub struct Hierarchy {
    root_folder: Folder
}

pub struct Folder {
    subfolders: HashMap<String, Folder>,
    snippets: Vec<usize>,
}

impl Hierarchy {
    pub fn new() -> Self {
        Hierarchy { root_folder: Folder::new() }
    }

    pub fn add_snippet<'a>(&mut self, snippet: usize, path: impl Iterator<Item=&'a str>) {
        let mut current_folder = &mut self.root_folder;

        for path_component in path {
            current_folder = current_folder.descend(path_component);
        }

        current_folder.add_snippet(snippet);
    }

    pub fn root(&self) -> &Folder {
        &self.root_folder
    }
}

impl Folder {
    pub fn new() -> Self {
        Folder { subfolders: HashMap::new(), snippets: Vec::new() }
    }

    pub fn descend<'a>(&mut self, name: &'a str) -> &mut Folder {
        if !self.subfolders.contains_key(name) {
            self.subfolders.insert(name.to_owned(), Folder::new());
        }

        self.subfolders.get_mut(name).unwrap()
    }

    pub fn add_snippet(&mut self, snippet: usize) {
        self.snippets.push(snippet);
    }

    pub fn subfolders(&self) -> impl Iterator<Item=(&String, &Folder)> {
        self.subfolders.iter()
    }

    pub fn snippets<'a>(&'a self) -> &'a Vec<usize> {
        &self.snippets
    }
}
