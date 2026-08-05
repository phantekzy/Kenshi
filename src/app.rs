use crate::anim::{Animation, Pulse};
use crate::tree::DirNode;

pub const VIEW_ANIM_MS: u64 = 380;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    Size,
    Name,
    Modified,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BottomPanel {
    Map,
    Types,
}

pub struct App {
    pub root: DirNode,
    pub nav_stack: Vec<usize>,
    selected_stack: Vec<usize>,
    pub selected: usize,
    pub sort_mode: SortMode,
    pub bottom_panel: BottomPanel,
    pub status: Option<String>,
    pub anim: Animation,
    pub pulse: Pulse,
}

impl App {
    pub fn new(root: DirNode) -> Self {
        Self {
            root,
            nav_stack: Vec::new(),
            selected_stack: Vec::new(),
            selected: 0,
            sort_mode: SortMode::Size,
            bottom_panel: BottomPanel::Map,
            status: None,
            anim: Animation::started(VIEW_ANIM_MS),
            pulse: Pulse::new(),
        }
    }

    fn restart_view_anim(&mut self) {
        self.anim = Animation::started(VIEW_ANIM_MS);
    }

    pub fn current_node(&self) -> &DirNode {
        let mut node = &self.root;
        for &i in &self.nav_stack {
            node = &node.children[i];
        }
        node
    }

    fn current_node_mut(&mut self) -> &mut DirNode {
        let mut node = &mut self.root;
        for &i in &self.nav_stack {
            node = &mut node.children[i];
        }
        node
    }

    pub fn breadcrumb(&self) -> String {
        let mut node = &self.root;
        let mut parts = vec![node.name.clone()];
        for &i in &self.nav_stack {
            node = &node.children[i];
            parts.push(node.name.clone());
        }
        parts.join(" / ")
    }

    pub fn move_down(&mut self) {
        let len = self.current_node().children.len();
        if len > 0 && self.selected + 1 < len {
            self.selected += 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn page_down(&mut self, page: usize) {
        let len = self.current_node().children.len();
        if len == 0 {
            return;
        }
        self.selected = (self.selected + page).min(len - 1);
    }

    pub fn page_up(&mut self, page: usize) {
        self.selected = self.selected.saturating_sub(page);
    }

    pub fn enter(&mut self) {
        let node = self.current_node();
        if self.selected >= node.children.len() {
            return;
        }
        if !node.children[self.selected].is_dir {
            return;
        }
        self.selected_stack.push(self.selected);
        self.nav_stack.push(self.selected);
        self.selected = 0;
        self.restart_view_anim();
    }

    pub fn back(&mut self) {
        if self.nav_stack.pop().is_some() {
            self.selected = self.selected_stack.pop().unwrap_or(0);
            self.restart_view_anim();
        }
    }

    pub fn toggle_sort(&mut self) {
        self.sort_mode = match self.sort_mode {
            SortMode::Size => SortMode::Name,
            SortMode::Name => SortMode::Modified,
            SortMode::Modified => SortMode::Size,
        };
        let mode = self.sort_mode;
        let node = self.current_node_mut();
        match mode {
            SortMode::Size => node.sort_by_size(),
            SortMode::Name => node.sort_by_name(),
            SortMode::Modified => node.sort_by_modified(),
        }
        self.selected = 0;
        self.restart_view_anim();
    }

    pub fn toggle_bottom_panel(&mut self) {
        self.bottom_panel = match self.bottom_panel {
            BottomPanel::Map => BottomPanel::Types,
            BottomPanel::Types => BottomPanel::Map,
        };
        self.restart_view_anim();
    }
}
