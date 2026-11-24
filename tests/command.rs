//! End-to-end tests for the Nostalgia command-line application.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_HOME_ID: AtomicU64 = AtomicU64::new(0);

struct TestHome(PathBuf);

impl TestHome {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "nostalgia-command-test-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_HOME_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join(".nostalgia")).unwrap();
        Self(path)
    }

    fn write_config(&self, contents: &str) {
        fs::write(self.0.join(".nostalgia/config.toml"), contents).unwrap();
    }

    fn run(&self, args: &[&str]) -> Output {
        self.run_with_path(args, None)
    }

    fn write_database(&self) {
        let image = self.0.join(".nostalgia/pokemon/fire/1.jpg");
        fs::create_dir_all(image.parent().unwrap()).unwrap();
        fs::write(image, b"test jpeg data").unwrap();

        let database = self.0.join(".nostalgia/database");
        fs::create_dir_all(&database).unwrap();
        fs::write(
            database.join("images.csv"),
            "category,subcategory,index,name,dark_threshold,background_color,foreground_color,type,second_type\n\
             pokemon,fire,1,charmander,0.5,112233,aabbcc,Fire,\n",
        )
        .unwrap();
    }

    fn write_fake_osascript(&self, status: i32) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;

        let bin = self.0.join("bin");
        fs::create_dir_all(&bin).unwrap();
        let program = bin.join("osascript");
        fs::write(
            &program,
            format!("#!/bin/sh\necho 'test osascript error' >&2\nexit {status}\n"),
        )
        .unwrap();
        fs::set_permissions(&program, fs::Permissions::from_mode(0o755)).unwrap();
        bin
    }

    fn write_empty_path(&self) -> PathBuf {
        let bin = self.0.join("empty-bin");
        fs::create_dir_all(&bin).unwrap();
        bin
    }

    fn run_with_path(&self, args: &[&str], path: Option<&PathBuf>) -> Output {
        let mut command = self.command(args);
        if let Some(path) = path {
            command.env("PATH", path);
        }
        command.output().unwrap()
    }

    fn run_with_xdg_config(&self, args: &[&str], xdg_config: &PathBuf) -> Output {
        self.command(args)
            .env("XDG_CONFIG_HOME", xdg_config)
            .output()
            .unwrap()
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_nostalgia"));
        command.args(args).env("HOME", &self.0);
        command.env_remove("XDG_CONFIG_HOME");
        command.env_remove("XDG_DATA_HOME");
        command
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn prints_the_configuration_location() {
    let home = TestHome::new();
    home.write_config("");

    let output = home.run(&["-v", "--get-config-location"]);

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains(&format!(
        "Config location: {}",
        home.0.join(".nostalgia").display()
    )));
}

#[test]
fn prints_the_package_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_nostalgia"))
        .arg("--version")
        .output()
        .unwrap();

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        format!("nostalgia {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn prints_the_loaded_configuration() {
    let home = TestHome::new();
    home.write_config("enabled_adaptors = []\ncategories = [\"plants\"]\n");

    let output = home.run(&["-v", "--print-config"]);

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("categories: [\n        \"plants\","));
}

#[test]
fn selects_a_random_database_row_when_no_adaptors_are_enabled() {
    let home = TestHome::new();
    home.write_config("enabled_adaptors = []\n");
    home.write_database();

    let output = home.run(&["-v"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("Category: pokemon"));
    assert!(stdout.contains("Subcategory: fire"));
    assert!(stdout.contains("Index: 1"));
}

#[test]
fn verbosity_flags_select_error_info_and_debug_output() {
    let home = TestHome::new();
    home.write_config("enabled_adaptors = []\n");
    home.write_database();

    let default_output = home.run(&[]);
    assert!(default_output.status.success());
    assert!(default_output.stdout.is_empty());

    let info_output = home.run(&["-v"]);
    assert!(info_output.status.success());
    let info_stdout = String::from_utf8_lossy(&info_output.stdout);
    assert!(info_stdout.contains("INFO"));
    assert!(info_stdout.contains("Category: pokemon"));
    assert!(!info_stdout.contains("Running DB!"));

    let debug_output = home.run(&["-vv"]);
    assert!(debug_output.status.success());
    let debug_stdout = String::from_utf8_lossy(&debug_output.stdout);
    assert!(debug_stdout.contains("DEBUG"));
    assert!(debug_stdout.contains("Running DB!"));
}

#[test]
fn validates_and_displays_all_database_rows_without_verbosity_flags() {
    let home = TestHome::new();
    home.write_config("enabled_adaptors = []\n");
    home.write_database();
    fs::write(
        home.0.join(".nostalgia/database/images.csv"),
        "category,subcategory,index,name,dark_threshold,background_color,foreground_color,type,second_type\n\
         pokemon,fire,1,charmander,0.5,112233,aabbcc,Fire,\n\
         pokemon,water,3,squirtle,0.4,223344,bbccdd,Water,\n\
         plants,forest,2,fern,0.3,223344,bbccdd,Grass,\n",
    )
    .unwrap();

    let output = home.run(&["--validate-database"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.contains("3 rows and 9 columns"));
    assert!(stdout.contains("charmander"));
    assert!(stdout.contains("fern"));
    assert!(stdout.contains("Category counts:"));
    assert!(stdout.contains("count"));
    assert!(stdout.contains("squirtle"));
}

#[test]
fn runs_ghostty_and_updates_its_user_configuration() {
    let home = TestHome::new();
    home.write_config("enabled_adaptors = [\"ghostty\"]\n");
    home.write_database();
    let xdg_config = home.0.join("xdg");
    let ghostty_config = xdg_config.join("ghostty/config");
    fs::create_dir_all(ghostty_config.parent().unwrap()).unwrap();
    fs::write(&ghostty_config, "font-size = 14\n").unwrap();

    let output = home.run_with_xdg_config(&["-v"], &xdg_config);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let updated = fs::read_to_string(ghostty_config).unwrap();
    assert!(updated.contains("font-size = 14"));
    assert!(updated.contains("background-image = "));
    assert!(updated.contains("background-opacity = 1.00"));
    assert!(updated.contains("background = 112233"));
    assert!(updated.contains("foreground = aabbcc"));
}

#[test]
fn creates_ghostty_configuration_when_it_does_not_exist() {
    let home = TestHome::new();
    home.write_config("enabled_adaptors = [\"ghostty\"]\n");
    home.write_database();

    let output = home.run(&[]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(home.0.join(".config/ghostty/config.ghostty").is_file());
}

#[test]
fn skips_ghostty_directories_when_finding_an_existing_configuration() {
    let home = TestHome::new();
    home.write_config("enabled_adaptors = [\"ghostty\"]\n");
    home.write_database();
    fs::create_dir_all(
        home.0
            .join("Library/Application Support/com.mitchellh.ghostty/config"),
    )
    .unwrap();
    let xdg_config = home.0.join(".config/ghostty/config");
    fs::create_dir_all(xdg_config.parent().unwrap()).unwrap();
    fs::write(&xdg_config, "font-size = 14\n").unwrap();

    let output = home.run(&[]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        fs::read_to_string(xdg_config)
            .unwrap()
            .contains("background-image")
    );
}

#[test]
fn reports_ghostty_configuration_path_errors() {
    let home = TestHome::new();
    home.write_config("enabled_adaptors = [\"ghostty\"]\n");
    home.write_database();
    fs::write(home.0.join(".config"), "not a directory").unwrap();

    let output = home.run(&[]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Error:"));
}

#[test]
fn runs_warp_and_writes_a_custom_theme_and_image() {
    let home = TestHome::new();
    home.write_config("enabled_adaptors = [\"warp\"]\n");
    home.write_database();

    let output = home.run(&[]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    #[cfg(target_os = "macos")]
    let themes = home.0.join(".warp/themes");
    #[cfg(all(unix, not(target_os = "macos")))]
    let themes = home.0.join(".local/share/warp-terminal/themes");
    assert_eq!(
        fs::read(themes.join("nostalgia.jpg")).unwrap(),
        b"test jpeg data"
    );
    let theme = fs::read_to_string(themes.join("nostalgia.yaml")).unwrap();
    assert!(theme.contains("background: '#112233'"));
    assert!(theme.contains("foreground: '#aabbcc'"));
    assert!(theme.contains("opacity: 100"));
}

#[test]
fn runs_darwin_wallpaper_adaptor_and_reports_command_failure() {
    let home = TestHome::new();
    home.write_config("enabled_adaptors = [\"darwin\"]\n");
    home.write_database();
    let bin = home.write_fake_osascript(0);

    let output = home.run_with_path(&["-v"], Some(&bin));

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("darwin: success"));

    let failed_bin = home.write_fake_osascript(1);
    let output = home.run_with_path(&[], Some(&failed_bin));

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("test osascript error"));
}

#[test]
fn reports_missing_osascript_for_system_adaptors() {
    let path_home = TestHome::new();
    path_home.write_config("enabled_adaptors = [\"darwin\"]\n");
    path_home.write_database();
    let empty_path = path_home.write_empty_path();
    let output = path_home.run_with_path(&[], Some(&empty_path));

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("No such file"));

    #[cfg(target_os = "macos")]
    {
        let terminal_home = TestHome::new();
        terminal_home.write_config("enabled_adaptors = [\"apple\"]\n");
        terminal_home.write_database();
        let empty_path = terminal_home.write_empty_path();
        let output = terminal_home.run_with_path(&[], Some(&empty_path));

        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("No such file"));
    }
}

#[cfg(target_os = "macos")]
#[test]
fn runs_apple_terminal_profile_color_adaptor() {
    let home = TestHome::new();
    home.write_config(
        "enabled_adaptors = [\"apple\"]\nchange_terminal_background_transparency = 50\n",
    );
    home.write_database();
    let bin = home.write_fake_osascript(0);

    let output = home.run_with_path(&["-v"], Some(&bin));

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .contains("Apple_Terminal: Apple Terminal profile colors updated")
    );

    let failed_bin = home.write_fake_osascript(1);
    let output = home.run_with_path(&[], Some(&failed_bin));

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("test osascript error"));
}

#[test]
fn executes_the_windows_adaptor_stub() {
    let home = TestHome::new();
    home.write_config("enabled_adaptors = [\"win32\"]\n");
    home.write_database();

    let output = home.run(&["-v"]);

    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("win32: success"));
}

#[test]
fn reports_database_errors_through_the_executable() {
    let home = TestHome::new();
    home.write_config("enabled_adaptors = []\n");

    let output = home.run(&[]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Error:"));
}
