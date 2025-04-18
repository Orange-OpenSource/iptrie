//! Graphviz representation of tries
//!
//! This module is only dedicated to produce graphviz representation
//! of the tries for debugging purpose.
//!
use std::io;
use std::io::Write;
use std::process::{Stdio, Command};
use std::fs::File;
use std::path::PathBuf;

const DOTCMD : &str = "dot";

/// Display the inner structure of tries (require `dot` and the activation of feature __graphviz__).
pub trait DotWriter {

    /// Writes the trie structure in dot format
    fn write_dot(&self, dot: &mut dyn Write) -> io::Result<()>;

    /// Generates the trie structure in a pdf file using `dot` command.
    ///
    /// If a file name is specified, the pdf file is generated.
    /// If not, the output is redirected to standard output.
    ///
    /// # Panics
    /// Panics if the `dot` command was not found.
    fn generate_pdf_file(&self, file: Option<&str>) -> io::Result<()>
    {
        let child = match file {
            None => {
                Command::new(DOTCMD)
                    .arg("-Tpdf")
                    .stdin(Stdio::piped())
                    .spawn()
            }
            Some(filename) => {
                let mut path = PathBuf::from(filename);
                path.set_extension("pdf");
                eprintln!("write output in file: {}", path.display());

                Command::new(DOTCMD)
                    .arg("-Tpdf")
                    .arg("-o").arg(path)
                    .stdin(Stdio::piped())
                    .spawn()
            }
        };
        let child = match child {
            Err(why) => panic!("couldn't spawn dot: {}", why),
            Ok(process) => process,
        };
        let mut dot = child.stdin.unwrap();
        self.write_dot(&mut dot)
    }

    /// Prints the trie structure in graphviz format.
    ///
    /// If a file name is specified, the graphviz file is generated.
    /// If not, the output is redirected to standard output.
    fn generate_graphviz_file(&self, file: Option<&str>) -> io::Result<()>
    {
        match file {
            None => {
                let mut dot = io::stdout();
                self.write_dot(&mut dot)
            }
            Some(filename) => {
                let mut path = PathBuf::from(filename);
                path.set_extension("gv");
                eprintln!("write output in file: {}", path.to_string_lossy());
                let mut dot = File::create(path)?;
                self.write_dot(&mut dot)
            }
        }
    }

    #[doc(hidden)]
    #[cfg(target_os = "macos")]
    fn open_dot_view(&self) -> io::Result<()>
    {
        use std::os::unix::io::AsRawFd;
        use std::os::unix::io::FromRawFd;

        let dot = match Command::new(DOTCMD)
            .arg("-Tpdf")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Err(why) => panic!("couldn't spawn dot: {}", why),
            Ok(process) => process,
        };
        // sur mac seulement...
        unsafe {
            Command::new("open")
                .arg("-f")
                .arg("-a").arg("Preview")
                .stdin(Stdio::from_raw_fd(dot.stdout.unwrap().as_raw_fd()))
                .spawn()?;
        }
        let mut dot = dot.stdin.unwrap();
        self.write_dot(&mut dot)
    }
}