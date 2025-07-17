use std::collections::HashSet;

/// Returns the rule paths for a single preset name.
pub fn preset_rules(name: &str) -> &'static [&'static str] {
    match name {
        "wordpress" => &[
            "/wp-login.php",
            "/cms/wp-includes/wlwmanifest.xml",
            "/xmlrpc.php",
            "/wp-json/wp/v2",
        ],
        "php" => &[
            "/phpmyadmin",
            "/admin.php",
            "/config.php",
            "/setup.php",
            "/test.php",
            "/dbadmin",
            "/mysql",
            "/pma",
            "/phpinfo.php",
            "/vendor/phpunit/phpunit/src/Util/PHP/eval-stdin.php",
        ],
        "config" => &[
            "/.env",
            "/env",
            "/config.json",
            "/config.yaml",
            "/config.inc.php",
        ],
        _ => &[],
    }
}

/// Generate a ruleset from a list of preset names.
pub fn from_preset_rules(presets: &[&str]) -> HashSet<String> {
    let mut set = HashSet::new();
    for preset in presets {
        for path in preset_rules(preset) {
            set.insert((*path).to_string());
        }
    }
    set
}

/// Generate a ruleset from a list of custom paths.
pub fn from_custom_rules(custom: &[&str]) -> HashSet<String> {
    custom.iter().map(|s| s.to_string()).collect()
}

/// Generate a ruleset from both presets and custom paths.
pub fn from_rules(presets: &[&str], custom: &[&str]) -> HashSet<String> {
    let mut set = from_preset_rules(presets);
    set.extend(from_custom_rules(custom));
    set
}
