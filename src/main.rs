#[macro_use] extern crate clap;
extern crate eve;
extern crate walkdir;

use std::fs;
use std::error;
use std::io::{self, Read};
use std::path::Path;

use eve::Eve;
use walkdir::WalkDir;

fn main() -> Result<(), Box<error::Error>> {
    let matches = clap_app!(eve =>
        (version: crate_version!())
        (author: "Aaron P. <theaaronepower@gmail.com> + Contributors")
        (about: crate_description!())
        (@arg inline: -i --inline requires[input]
            "Edit file(s) in place, saving backups with the\
            specified extension.")
        (@arg extension: -e --extension +takes_value requires[inline]
            "Extension to save backups of files edited with `-i`.\
            If not present no backup is saved.")
        (@arg file: -f --file +takes_value
            "Use specified environment file over default `.env`.")
        (@arg recursive: -r --recursive
            "Perform a recursive search and replace on a directory.")
        (@arg input: ... "The files/directories to search and replace.")
    ).get_matches();

    let inline_option = matches.is_present("inline");
    let file_option = matches.value_of("file");
    let recursive_option = matches.is_present("recursive");
    let extension_option = matches.value_of("extension");
    let paths: Vec<&str> = matches.values_of("input").unwrap().collect();

    let mut eve = if let Some(env) = file_option {
        Eve::from_path(env).unwrap()
    } else {
        Eve::new().map_err(|_| "No `.env` present.").unwrap()
    };

    if paths.len() == 0 {
        let text = {
            let mut b = String::new();
            io::stdin().read_to_string(&mut b)?;
            b
        };

        let replaced = eve.replace(&text)?;
        println!("{}", replaced);
        return Ok(())
    }

    macro_rules! replace {
        ($path:expr) => {{
            let original = fs::read_to_string($path)?;
            let replaced = eve.replace(&original)?;

            if inline_option {
                if let Some(ext) = extension_option {
                    fs::write(Path::new($path).with_extension(ext), &original)?;
                }

                fs::write($path, replaced.as_bytes())?;
            } else {
                println!("{}", replaced);
            }
        }}
    }

    for path in paths {
        let is_dir = fs::metadata(path)?.is_dir();
        if is_dir && recursive_option {
            let walker = WalkDir::new(path)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file());

            for entry in walker {
                replace!(entry.path());
            }

        } else if is_dir && !recursive_option {
            return Err("Directory provided without --recursive flag.".into())
        } else {
            replace!(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate assert_cli;
    extern crate dir_diff;
    extern crate tempfile;

    use std::io::Write;
    use std::fs;

    use self::assert_cli::Assert;
    use self::tempfile::{TempDir, NamedTempFile};

    #[test]
    fn single_file() {
        let mut env_file = NamedTempFile::new().unwrap();
        let mut test_file = NamedTempFile::new().unwrap();

        write!(env_file, "HELLO=Hello").unwrap();
        writeln!(test_file, "{{{{HELLO}}}} World!").unwrap();

        Assert::main_binary()
            .with_args(&[
                       "-f",
                       env_file.path().to_str().unwrap(),
                       test_file.path().to_str().unwrap()
            ])
            .succeeds()
            .stdout()
            .is("Hello World!\n")
            .unwrap()
    }

    #[test]
    fn backup_inline_file() {
        const EXTENSION: &str = "backup";
        const ORIGINAL: &[u8] = b"{{HELLO}} World!";
        let mut env_file = NamedTempFile::new().unwrap();
        let test_file = NamedTempFile::new().unwrap();

        write!(env_file, "HELLO=Hello").unwrap();
        fs::write(test_file.path(), ORIGINAL).unwrap();

        Assert::main_binary()
            .with_args(&[
                       "-i",
                       "-e",
                       EXTENSION,
                       "-f",
                       env_file.path().to_str().unwrap(),
                       test_file.path().to_str().unwrap()
            ])
            .succeeds()
            .stdout()
            .is("")
            .unwrap();

        let backup_path = test_file.path().with_extension(EXTENSION);

        assert_eq!("Hello World!", fs::read_to_string(test_file.path()).unwrap());
        assert_eq!(&ORIGINAL, &&*fs::read(backup_path).unwrap());
    }

    #[test]
    fn two_files() {
        let mut env_file = NamedTempFile::new().unwrap();
        let mut first_file = NamedTempFile::new().unwrap();
        let mut second_file = NamedTempFile::new().unwrap();

        write!(env_file, "HELLO=Hello").unwrap();
        write!(first_file, "{{{{HELLO}}}} World!").unwrap();
        write!(second_file, "World! {{{{HELLO}}}}").unwrap();

        Assert::main_binary()
            .with_args(&[
                "-f",
                env_file.path().to_str().unwrap(),
                first_file.path().to_str().unwrap(),
                second_file.path().to_str().unwrap()
            ])
            .succeeds()
            .stdout()
            .is("Hello World!\nWorld! Hello")
            .unwrap();
    }

    #[test]
    fn single_inline_file() {
        let mut env_file = NamedTempFile::new().unwrap();
        let mut test_file = NamedTempFile::new().unwrap();

        write!(env_file, "HELLO=Hello").unwrap();
        write!(test_file, "{{{{HELLO}}}} World!").unwrap();

        Assert::main_binary()
            .with_args(&[
                       "-i",
                       "-f",
                       env_file.path().to_str().unwrap(),
                       test_file.path().to_str().unwrap()
            ])
            .succeeds()
            .stdout()
            .is("")
            .unwrap();

        assert_eq!("Hello World!", fs::read_to_string(test_file.path()).unwrap())
    }

    #[test]
    fn directory() {
        let mut env_file = NamedTempFile::new().unwrap();
        let expected_dir = TempDir::new().unwrap();
        let actual_dir = TempDir::new().unwrap();
        let file_contents = &[
            ("Hello World!", "{{HELLO}} World!"),
            ("Hello World!", "Hello {{WORLD}}")
        ];
        // Vec to retain temp files till the end of the function.
        let mut file_vec = Vec::with_capacity(file_contents.len());

        for (expected, actual) in file_contents {
            let expected_file = NamedTempFile::new_in(expected_dir.path()).unwrap();
            let actual_file = NamedTempFile::new_in(actual_dir.path()).unwrap();

            fs::write(expected_file.path(), expected.as_bytes()).unwrap();
            fs::write(actual_file.path(), actual.as_bytes()).unwrap();

            file_vec.push((expected_file, actual_file));
        }

        write!(env_file, "HELLO=Hello\nWORLD=World!").unwrap();

        Assert::main_binary()
            .with_args(&[
                       "-ir",
                       "-f",
                       env_file.path().to_str().unwrap(),
                       actual_dir.path().to_str().unwrap()
            ])
            .succeeds()
            .stdout()
            .is("")
            .unwrap();

        assert!(dir_diff::is_different(expected_dir.path(), actual_dir.path()).unwrap());
    }
}
