use serde::{Deserialize, Serialize};
use std::default::Default;
use std::path::{Path, PathBuf};

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

impl Group {
    // TODO fillvalue can be any numeric type
    // fillvalue: Option<f64>
    fn create_dataset(&self, name: &str) -> Dataset {
        unimplemented!();
    }
    fn create_group(&self, name: &str) -> Group {
        unimplemented!();
    }
}

#[derive(Debug)]
struct File;

impl File {}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum ObjectType {
    Dataset,
    Group,
    File,
}

#[derive(Debug, Deserialize, Serialize)]
struct InnerMarker {
    #[serde(rename = "type")]
    objtype: ObjectType,
    version: u8,
}

#[derive(Debug, Deserialize, Serialize)]
struct Marker {
    exdir: InnerMarker,
}

impl Marker {
    fn new(objtype: ObjectType) -> Self {
        Marker {
            exdir: InnerMarker {
                objtype,
                version: 1,
            },
        }
    }
}

impl Default for Marker {
    fn default() -> Self {
        Marker::new(ObjectType::Dataset)
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
        testdir: PathBuf,
        testfilep: PathBuf,
        testfile: Option<File>,
    }

    #[fixture]
    fn setup_teardown_folder() -> FixtureExdir {
        let name = Uuid::new_v4().to_string();
        let testpathbase = TempDir::new(name.as_str()).unwrap();
        let testdir = testpathbase.path().join("exdir_dir");
        let testfilep = testpathbase.path().join("test.exdir");
        create_dir_all(testdir.clone()).unwrap();
        FixtureExdir {
            testpathbase,
            testdir,
            testfilep,
            testfile: None,
        }
    }

    // #[fixture]
    // fn exdir_tmpfile() {}

    #[rstest]
    fn object_init(setup_teardown_folder: FixtureExdir) {
        let obj = Object::new(
            setup_teardown_folder.testdir.as_path(),
            Path::new(""),
            "test_object",
            None,
        );
        assert_eq!(obj.root_directory, setup_teardown_folder.testdir);
        assert_eq!(obj.object_name, "test_object".to_string());
        assert_eq!(obj.parent_path, PathBuf::from(""));
        assert!(obj.file.is_none());
        assert_eq!(obj.relative_path, PathBuf::from("test_object"));
        assert_eq!(obj.name, PathBuf::from("/test_object"));
    }

    #[rstest]
    fn open_object() {}

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

        // println!("{:02x?}", out_buf);
        // println!("{:?}", out_buf);
        Ok(())
    }

    // metadata
    const EXDIR_METANAME: &str = "exdir";
    const TYPE_METANAME: &str = "type";
    const VERSION_METANAME: &str = "version";

    // filenames
    const META_FILENAME: &str = "exdir.yaml";
    const ATTRIBUTES_FILENAME: &str = "attributes.yaml";
    const RAW_FOLDER_NAME: &str = "__raw__";

    // typenames
    const DATASET_TYPENAME: &str = "dataset";
    const GROUP_TYPENAME: &str = "group";
    const FILE_TYPENAME: &str = "file";

    #[test]
    fn yaml_example() -> Result<(), serde_yaml::Error> {
        println!(
            "{}",
            serde_yaml::to_string(&Marker::new(ObjectType::Dataset))?
        );
        println!("{}", serde_yaml::to_string(&Marker::new(ObjectType::File))?);
        println!(
            "{}",
            serde_yaml::to_string(&Marker::new(ObjectType::Group))?
        );
        Ok(())
    }
}
