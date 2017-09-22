use ::color::Color;
use ::powerline::*;
use ::part::*;
use std::process::Command;
use std::str;

pub struct GitInfo {
    untracked: u32,
    conflicted: u32,
    non_staged: u32,
    pub ahead: u32,
    pub behind: u32,
    staged: u32,
}

impl GitInfo{
    pub fn new() -> GitInfo{
        GitInfo{ untracked: 0, conflicted: 0, non_staged: 0, staged: 0, ahead: 0, behind: 0}
    }

    fn is_dirty(&self) -> bool {
        (self.untracked + self.conflicted + self.staged + self.non_staged) > 0
    }

    fn add_file(&mut self, begin: &str) -> Result<(), Error> {
        match begin{
            "??" => self.untracked += 1,
            "DD" => self.conflicted += 1,
            "AU" => self.conflicted += 1,
            "UD" => self.conflicted += 1,
            "UA" => self.conflicted += 1,
            "UU" => self.conflicted += 1,
            "DU" => self.conflicted += 1,
            "AA" => self.conflicted += 1,
            _ => {
                let mut chars = begin.chars();
                let a = chars.next().ok_or(Error::from_str("Invalid file status"))?;
                let b = chars.next().ok_or(Error::from_str("Invalid file status"))?;
                if b != ' '{ self.non_staged += 1; }
                if a != ' '{ self.staged += 1; }
            },
        };
        Ok(())
    }

}

fn get_detached_branch_name() -> Result<String, Error> {
    let child = Command::new("git").args(&["describe", "--tags", "--always"]).output().map_err(|e| Error::wrap(e, "Failed to run git"))?;
    Ok(if !child.status.success() {
        String::from("Big Bang")
    } else {
        let branch = str::from_utf8(&child.stdout)?.split("\n").next().ok_or(Error::from_str("Empty git output"))?;
        format!("\u{2693}{}", branch)
    })
}

fn quantity(val: u32) -> String{
    if val  > 1 {
        format!("{}",val)
    } else {
        String::from("")
    }
}

fn get_first_number(s: &str) -> u32 {
    let mut value = 0;
    for c in s.chars() {
        if let Some(x) = c.to_digit(10) {
            value = value * 10 + x;
        }
        else { return value; }
    }
    value
}

fn get_ahead_commits(s: &str) -> Option<u32> {
    if let Some(pos) = s.find("ahead") {
        let start = pos + 6;
        let rest = s.get(start..).unwrap();
        return Some(get_first_number(rest));
    }
    None
}

fn get_behind_commits(s: &str) -> Option<u32> {
    if let Some(pos) = s.find("behind") {
        let start = pos + 7;
        let rest = s.get(start..).unwrap();
        return Some(get_first_number(rest));
    }
    None
}

fn get_branch_name<'a>(s: &'a str) -> Option<&'a str> {
    if let Some(rest) = s.get(3..) {
        let mut end = 0usize;
        let mut dot_count = 0;
        let mut text = rest.chars();
        
        while let Some(c) = text.next() {
            end += 1;
            if c.is_whitespace() {
                if let Some(next) = text.next() {
                    if next != '[' { return None }
                }
                break;
            }
            if c == '.' {
                dot_count += 1;
                if dot_count == 3 { end -= 3; break; }
            }

        }
        rest.get(..end)
    }
    else {
        None
    }
}
impl Part for GitInfo {
    fn get_segments(mut self) -> Result<Vec<Segment>, Error> {
        let output = Command::new("git").args(&["status", "--porcelain", "-b"]).output().map_err(|e| Error::wrap(e, "Failed to run git"))?;

        let data = &output.stdout;
        if data.len() == 0 { return Ok(Vec::new()); }
        let mut lines = data.split(|x| *x == ('\n' as u8));

        let branch_line = str::from_utf8(lines.next().ok_or(Error::from_str("Empty git output"))?)?;
        
        let branch = {
            if let Some(branch_search) = get_branch_name(&branch_line) {
                if let Some(pos) = branch_line.find('[') {
                    let info = branch_line.get(pos..).unwrap();
                    self.ahead += get_ahead_commits(&info).unwrap_or(0);
                    self.behind += get_behind_commits(&info).unwrap_or(0); 
                }
                String::from(branch_search)
            }
            else {
                 get_detached_branch_name()?
            }
        };
        
        for line in lines {
            if let Some(op) = line.get(..2) {
                self.add_file(str::from_utf8(op)?)?;
            }
        }

        let (branch_fg, branch_bg) = if self.is_dirty() {
            (Color::REPO_DIRTY_FG, Color::REPO_DIRTY_BG)
        } else {
            (Color::REPO_CLEAN_FG, Color::REPO_CLEAN_BG)
        };
        let mut segments = Vec::new();
        segments.push(Segment::simple(&format!(" {} ", branch), branch_fg, branch_bg));
        {
            let mut add_elem = |count, symbol, fg, bg| {
                if count > 0 {
                    let text = format!(" {}{} ", quantity(count), symbol);
                    segments.push(Segment::simple(&text, fg, bg));
                }
            };
            add_elem(self.ahead, '\u{2B06}', Color::GIT_AHEAD_FG, Color::GIT_AHEAD_BG);
            add_elem(self.behind, '\u{2B07}', Color::GIT_BEHIND_FG, Color::GIT_BEHIND_BG);
            add_elem(self.staged, '\u{2714}', Color::GIT_STAGED_FG, Color::GIT_STAGED_BG);
            add_elem(self.non_staged, '\u{270E}', Color::GIT_NOTSTAGED_FG, Color::GIT_NOTSTAGED_BG);
            add_elem(self.untracked, '\u{2753}', Color::GIT_UNTRACKED_FG, Color::GIT_UNTRACKED_BG);
            add_elem(self.conflicted, '\u{273C}', Color::GIT_CONFLICTED_FG, Color::GIT_CONFLICTED_BG);
        }
        Ok(segments)
    }
}
