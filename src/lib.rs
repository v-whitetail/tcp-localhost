#![allow(unused,dead_code)]

pub mod cli;
pub mod utils {

    use std::sync::mpsc::channel;
    use std::path::{ Path, PathBuf, Component, };

    use clap::Parser;
    use anyhow::{Result, bail, anyhow};
    use itertools::any;
    use native_dialog::MessageDialog;

    use live_server::listen;
    use tokio::task::spawn_blocking;
    use notify::{
        Event,
        Config,
        Watcher,
        EventKind,
        RecursiveMode,
        RecommendedWatcher,
        event::CreateKind,
        event::RemoveKind,
    };

    use crate::homepage;



    pub async fn host(ip: &str, port: &u16, root: &PathBuf) -> Result<()> {
        listen(ip, *port, root).await?;
        Ok(())
    }




    pub async fn watch_dir(root: &PathBuf) -> Result<()> {

        let (tx,rx) = channel();
        let mut watcher = RecommendedWatcher::new(tx,Config::default())?;

        let templates = root.join("Templates");
        let reports = root.join("Reports");

        watcher.watch(&templates, RecursiveMode::NonRecursive);
        watcher.watch(&reports, RecursiveMode::NonRecursive);

        let root = root.to_owned();

        let dir_watch = spawn_blocking( move ||{
            rx.iter()
                .filter_map(|event| event.ok())
                .for_each(|event|{
                    let status = match (event.kind) {
                        EventKind::Create(_) => handle_file_created(&event,&root),
                        EventKind::Remove(_) => handle_file_removed(&event,&root),
                        _ => Ok(()),
                    };
                    if let Err(error) = status{
                        MessageDialog::new()
                            .set_type(native_dialog::MessageType::Error)
                            .set_title("tcp_localhost.exe")
                            .set_text(&error.to_string())
                            .show_alert()
                            .unwrap();
                    };
                });
        }).await?;
        Ok(())
    }




    pub fn is_templates(path: &PathBuf) -> bool {
        any(
            path.components(),
            |component| component.as_os_str() == "Templates") 
    }

    pub fn is_reports(path: &PathBuf) -> bool {
        any(
            path.components(),
            |component| component.as_os_str() == "Reports"
           ) 
    }

    pub fn is_html(path: &PathBuf) -> bool {
        if let Some(extension) = path.extension() {
            extension == "html"
        } else { false }
    }

    pub fn clone_to_reports(path: &PathBuf) -> Result<()> {
        if is_templates(&path)
            && is_html(&path) {
                let template_path = path;
                let report_path = PathBuf::from(
                    template_path.to_string_lossy().replace("Templates", "Reports")
                    );
                std::fs::copy(template_path, report_path)?;
            }
        Ok(())
    }

    pub fn remove_cloned_report(path: &PathBuf) -> Result<()> {
        if is_templates(&path)
            && is_html(&path) {
                let template_path = path;
                let report_path = PathBuf::from(
                    template_path.to_string_lossy().replace("Templates", "Reports")
                    );
                std::fs::remove_file(report_path)?;
            }
        Ok(())
    }

    fn handle_file_created(event: &Event, root: &PathBuf) -> Result<()> {
        event
            .to_owned()
            .paths
            .into_iter()
            .try_for_each(|path| clone_to_reports(&path) )?;
        homepage::update_links(root)?;
        Ok(())
    }

    fn handle_file_removed(event: &Event, root: &PathBuf) -> Result<()> {
        event
            .to_owned()
            .paths
            .into_iter()
            .try_for_each(|path| remove_cloned_report(&path) )?;
        homepage::update_links(root)?;
        Ok(())
    }





    pub fn startup(root: &PathBuf) -> Result<()> {

        use std::fs::{read_dir,create_dir,remove_file,copy,write};

        let index = root.join("index.html");
        let reports = root.join("Reports");
        let templates = root.join("Templates");
        let resources = root.join("Resources");
        let backup = resources.join("index.bak");

        let required_dirs = [
            &templates,
            &resources,
            &reports,
        ];

        required_dirs
            .into_iter()
            .try_for_each(|item|
                          if !item.exists() && item.is_dir() { create_dir(item) }
                          else { Ok(()) }
                         )?;

        if !index.exists() {
            if backup.exists() {
                copy(backup,index)?;
            } else {
                write(index, homepage::EMPTY_INDEX)?;
            }
        }

        read_dir(reports)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .try_for_each(|filepath| remove_file(filepath))?;

        read_dir(templates)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .try_for_each(|filepath| clone_to_reports(&filepath))?;

        homepage::update_links(root)?;

        Ok(())
    }


}

pub mod homepage {

    use crate::utils;
    use anyhow::{Result,ensure};
    use itertools::Itertools;
    use std::fs::{read_to_string,write,read_dir};
    use std::path::{Path,PathBuf};

    pub const TEMPLATE_LINE: &str = r#"
        <a class="link-row" href="Templates/tdoc.html">tdoc</a>
        "#;
    pub const REPORT_LINE: &str = r#"
        <a class="link-row" href="Reports/rdoc.html">rdoc</a>
        "#;

    pub const EMPTY_INDEX: &str = r#"
        <html> <head> </head>
        <body> <div> <span>
        default index.html
        </span> </div>
        <div> <span>
        please replace $IPATH/index.html with proper home page
        </span> </div> </body> </html>
        "#;


    pub fn reader<P: AsRef<Path>>(path: P) -> Result<String> {
        Ok(read_to_string(path)?)
    }
    
    fn writer<P: AsRef<Path>>(path: P, buffer: &str) -> Result<()> {
        Ok(write(path, buffer)?)
    }





    pub fn update_links(root: &PathBuf) -> Result<()> {
        let (tdir, rdir) = (root.join("Templates"), root.join("Reports"));

        let home_page = reader(root.join("index.html"))?;

        let mut link_blocks = home_page
            .split(r#"<div class="link-block">"#)
            .skip(1).take(2)
            .filter_map(|link_block| link_block.split_once(r#"</h1>"#))
            .map(|(_,outer_text)| outer_text)
            .filter_map(|link_block| link_block.split_once(r#"</div>"#))
            .map(|(inner_text,_)| inner_text);

        let template_lines = link_blocks.nth(0).unwrap_or(TEMPLATE_LINE);
        let report_lines = link_blocks.nth(0).unwrap_or(REPORT_LINE);

        let templates = read_dir(tdir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| utils::is_html(path));
            
        let reports = read_dir(rdir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| utils::is_html(path));

        let template_links = templates
            .filter_map(|path| replace_with_stem(TEMPLATE_LINE,"tdoc",path))
            .sorted()
            .collect::<String>();

        let report_links = reports
            .filter_map(|path| replace_with_stem(REPORT_LINE,"rdoc",path))
            .sorted()
            .collect::<String>();

        ensure!(!template_links.is_empty(),"empty Templates");
        ensure!(!report_links.is_empty(),"empty Reports");

        if template_links!=template_lines || report_lines!=report_links {
            let updated_home_page = home_page
                .replace(&template_lines, &template_links)
                .replace(&report_lines, &report_links);
            writer(root.join("index.html"), &updated_home_page)
        } else { Ok(()) }

    }

    fn replace_with_stem(s: &str, p: &str, file: PathBuf) -> Option<String> {
        file.file_stem()
            .and_then(|stem| Some(
                    s.replace(p,&stem.to_string_lossy())
                    ))
    }

}
