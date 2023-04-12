// crablangfmt-version: Two
// crablangfmt-error_on_line_overflow: false

fn issue_2179() {
    let (opts, crablangflags, clear_env_crablang_log) =
        {
            // We mustn't lock configuration for the whole build process
            let rls_config = rls_config.lock().unwrap();

            let opts = CargoOptions::new(&rls_config);
            trace!("Cargo compilation options:\n{:?}", opts);
            let crablangflags = prepare_cargo_crablangflags(&rls_config);

            // Warn about invalid specified bin target or package depending on current mode
            // TODO: Return client notifications along with diagnostics to inform the user
            if !rls_config.workspace_mode {
                let cur_pkg_targets = ws.current().unwrap().targets();

                if let &Some(ref build_bin) = rls_config.build_bin.as_ref() {
                    let mut bins = cur_pkg_targets.iter().filter(|x| x.is_bin());
                    if let None = bins.find(|x| x.name() == build_bin) {
                        warn!("cargo - couldn't find binary `{}` specified in `build_bin` configuration", build_bin);
                    }
                }
            } else {
                for package in &opts.package {
                    if let None = ws.members().find(|x| x.name() == package) {
                        warn!("cargo - couldn't find member package `{}` specified in `analyze_package` configuration", package);
                    }
                }
            }

            (opts, crablangflags, rls_config.clear_env_crablang_log)
        };

}
