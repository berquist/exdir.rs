use serde::{Deserialize, Serialize};
use std::default::Default;
use std::fs::{create_dir_all, remove_dir_all};
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

// metadata
const EXDIR_METANAME: &str = "exdir";
const TYPE_METANAME: &str = "type";
const VERSION_METANAME: &str = "version";

// filenames
const META_FILENAME: &str = "exdir.yaml";
const ATTRIBUTES_FILENAME: &str = "attributes.yaml";
const RAW_FOLDER_NAME: &str = "__raw__";

// typenames
// const DATASET_TYPENAME: &str = "dataset";
// const GROUP_TYPENAME: &str = "group";
// const FILE_TYPENAME: &str = "file";

struct Object {
    root_directory: PathBuf,
    object_name: String,
    parent_path: PathBuf,
    relative_path: PathBuf,
    relative_name: String,
    name: PathBuf,
    file: Option<std::fs::File>,
}

impl Object {
    fn new(
        root_directory: &Path,
        parent_path: &Path,
        object_name: &str,
        file: Option<std::fs::File>,
    ) -> Self {
        let object_name = String::from(object_name);
        let relative_path = parent_path.join(object_name.clone());
        let mut relative_name = String::from(relative_path.to_str().unwrap());
        if relative_name == "." {
            relative_name = String::from("");
        }
        let name = PathBuf::from("/").join(relative_name.clone());
        Object {
            root_directory: root_directory.to_path_buf(),
            object_name,
            parent_path: parent_path.to_path_buf(),
            relative_path,
            relative_name,
            name,
            file,
        }
    }
}

#[derive(Debug)]
struct Group;

#[derive(Debug)]
struct Dataset;

trait HasLeaves {
    fn create_dataset(&self, name: &str) -> Dataset;
    fn create_group(&self, name: &str) -> Group;
}

impl HasLeaves for Group {
    // TODO fillvalue can be any numeric type
    // fillvalue: Option<f64>
    fn create_dataset(&self, name: &str) -> Dataset {
        Dataset
    }

    fn create_group(&self, name: &str) -> Group {
        Group
    }
}

enum OpenMode {
    ReadWrite,
    ReadOnly,
    FileClosed,
}

const RECOGNIZED_MODES: [&str; 7] = ["a", "r", "r+", "w", "w-", "x", "a"];

// enum NamingRule {
//     Simple,
//     Strict,
//     Thorough,
//     None,
// }

#[derive(Debug)]
struct File;

fn _create_object_directory(directory: &PathBuf, metadata: &Metadata) {
    if directory.exists() {
        eprintln!("The directory '{:?}' already exists", directory);
        panic!();
    }
    create_dir_all(directory.as_path()).unwrap();
    let meta_filename = directory.join(META_FILENAME);
    serde_yaml::to_writer(
        BufWriter::new(std::fs::File::create(meta_filename.as_path()).unwrap()),
        metadata,
    )
    .unwrap();
}

fn is_nonraw_object_directory(directory: &PathBuf) -> bool {
    let meta_filename = directory.join(META_FILENAME);
    if !meta_filename.exists() {
        return false;
    }
    let _meta_data: Metadata = serde_yaml::from_reader(BufReader::new(
        std::fs::File::open(meta_filename.as_path()).unwrap(),
    ))
    .unwrap();
    true
}

impl File {
    fn new(
        directory: &str,
        mode: Option<&str>,
        allow_remove: Option<bool>,
    ) -> Result<Self, std::io::Error> {
        let allow_remove = allow_remove.unwrap_or(false);

        let mode = mode.unwrap_or("a");
        if !RECOGNIZED_MODES.contains(&mode) {
            eprintln!(
                "IO mode {} not recognized, mode must be one of {:?}",
                mode, RECOGNIZED_MODES
            );
            panic!();
        }

        let directory = PathBuf::from(directory);
        let target_ext = ".exdir";
        let directory = match directory.extension() {
            None => directory.join(target_ext),
            Some(ext) => {
                if ext != target_ext {
                    directory.join(ext)
                } else {
                    directory
                }
            }
        };

        // no plugins in this implementation

        // no (customizable) name validation in this implementation

        let already_exists = directory.exists();
        if already_exists {
            if !is_nonraw_object_directory(&directory) {
                eprintln!(
                    "Path '{:?}' already exists, but is not a valid exdir file.",
                    directory
                );
                panic!();
            }
        }

        let mut should_create_directory = false;

        match mode {
            "r" => {
                if !already_exists {
                    panic!()
                }
            }
            "r+" => {
                if !already_exists {
                    panic!()
                }
            }
            "w" => {
                if already_exists {
                    if allow_remove {
                        remove_dir_all(&directory)?;
                    } else {
                        panic!()
                    }
                }
                should_create_directory = true;
            }
            "w-" | "x" => {
                if already_exists {
                    panic!()
                }
                should_create_directory = true;
            }
            "a" => {
                if !already_exists {
                    should_create_directory = true;
                }
            }
            _ => panic!(),
        }

        if should_create_directory {
            // TODO self.name_validation(directory.parent, directory.name)
            _create_object_directory(&directory, &Metadata::new(ObjectType::File));
        }

        Ok(File {})
    }

    fn default(directory: &str) -> Self {
        Self::new(directory, None, Some(false)).unwrap()
    }
}

impl HasLeaves for File {
    fn create_dataset(&self, name: &str) -> Dataset {
        Dataset
    }

    fn create_group(&self, name: &str) -> Group {
        Group
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum ObjectType {
    Dataset,
    Group,
    File,
}

#[derive(Debug, Deserialize, Serialize)]
struct InnerMetadata {
    #[serde(rename = "type")]
    objtype: ObjectType,
    version: u8,
}

#[derive(Debug, Deserialize, Serialize)]
struct Metadata {
    exdir: InnerMetadata,
}

impl Metadata {
    fn new(objtype: ObjectType) -> Self {
        Metadata {
            exdir: InnerMetadata {
                objtype,
                version: 1,
            },
        }
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata::new(ObjectType::Dataset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use npyz::WriterBuilder;
    use rstest::{fixture, rstest};
    use std::fs::create_dir_all;
    use tempdir::TempDir;
    use uuid::Uuid;

    #[derive(Debug)]
    struct FixtureExdir {
        testpathbase: TempDir,
        testdir: Option<PathBuf>,
        testfilep: Option<PathBuf>,
        testfile: Option<File>,
    }

    fn make_tempdir() -> TempDir {
        let name = Uuid::new_v4().to_string();
        TempDir::new(name.as_str()).unwrap()
    }

    #[fixture]
    fn setup_teardown_folder() -> FixtureExdir {
        let testpathbase = make_tempdir();
        let testdir = testpathbase.path().join("exdir_dir");
        let testfilep = testpathbase.path().join("test.exdir");
        create_dir_all(testdir.clone()).unwrap();
        FixtureExdir {
            testpathbase,
            testdir: Some(testdir),
            testfilep: Some(testfilep),
            testfile: None,
        }
    }

    #[fixture]
    fn setup_teardown_file() -> FixtureExdir {
        let testpathbase = make_tempdir();
        let testdir = testpathbase.path().join("exdir_dir");
        let testfilep = testpathbase.path().join("test.exdir");
        create_dir_all(testdir.clone()).unwrap();
        let testfile = Some(File::new(testfilep.to_str().unwrap(), Some("w"), None).unwrap());
        FixtureExdir {
            testpathbase,
            testdir: Some(testdir),
            testfilep: Some(testfilep),
            testfile,
        }
    }

    #[fixture]
    fn exdir_tmpfile() -> FixtureExdir {
        let testpathbase = make_tempdir();
        let testfilep = Some(testpathbase.path().join("test.exdir"));
        FixtureExdir {
            testpathbase,
            testdir: None,
            testfilep,
            testfile: Some(File::new("", Some("w"), None).unwrap()),
        }
    }

    // #[fixture]
    // fn hdf5_tmpfile() -> FixtureHdf5 {}

    #[rstest]
    fn object_init(setup_teardown_folder: FixtureExdir) {
        let tdir = setup_teardown_folder.testdir.unwrap();
        let obj = Object::new(tdir.as_path(), Path::new(""), "test_object", None);
        assert_eq!(obj.root_directory, tdir);
        assert_eq!(obj.object_name, "test_object".to_string());
        assert_eq!(obj.parent_path, PathBuf::from(""));
        assert!(obj.file.is_none());
        assert_eq!(obj.relative_path, PathBuf::from("test_object"));
        assert_eq!(obj.name, PathBuf::from("/test_object"));
    }

    #[rstest]
    fn open_object(exdir_tmpfile: FixtureExdir) {
        let grp = exdir_tmpfile.testfile.unwrap().create_group("test");
        let _grp2 = grp.create_group("test2");
    }

    #[rstest]
    fn object_attrs(setup_teardown_file: FixtureExdir) {}

    #[rstest]
    fn object_meta(setup_teardown_file: FixtureExdir) {}

    #[rstest]
    fn object_directory(setup_teardown_file: FixtureExdir) {}

    #[rstest]
    fn object_create_raw(setup_teardown_file: FixtureExdir) {}

    #[test]
    fn npy_example() -> std::io::Result<()> {
        let mut out_buf = vec![];
        let mut writer = npyz::WriteOptions::new()
            .default_dtype()
            .shape(&[2, 3])
            .writer(&mut out_buf)
            .begin_nd()?;
        writer.push(&100)?;
        writer.push(&101)?;
        writer.push(&102)?;
        writer.extend(vec![200, 201, 202])?;
        writer.finish()?;

        // There's extra stuff at the beginning because the writer puts the
        // header in.
        // println!("{:02x?}", out_buf);
        // println!("{:?}", out_buf);
        Ok(())
    }

    #[test]
    fn yaml_example() -> Result<(), serde_yaml::Error> {
        println!(
            "{}",
            serde_yaml::to_string(&Metadata::new(ObjectType::Dataset))?
        );
        println!(
            "{}",
            serde_yaml::to_string(&Metadata::new(ObjectType::File))?
        );
        println!(
            "{}",
            serde_yaml::to_string(&Metadata::new(ObjectType::Group))?
        );
        Ok(())
    }
}
