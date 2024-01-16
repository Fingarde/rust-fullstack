use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use futures::channel::mpsc::{channel, Receiver};
use pathdiff::diff_paths;
use futures::{SinkExt, StreamExt};
use std::path::Path;
use std::sync::Arc;
use shells::bash;
use tera::Tera;
use tokio::sync::broadcast::Sender;
use tokio::sync::RwLock;

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Default::default(),
    )?;

    Ok((watcher, rx))
}

pub fn watch(sender: Sender<bool>, tera: Arc<RwLock<Tera>>) {
    tokio::spawn(async move {
        let (mut watcher, mut rx) = async_watcher().unwrap();

        watcher
            .watch("./src/view".as_ref(), RecursiveMode::Recursive)
            .unwrap();

        while let Some(res) = rx.next().await {
            if let Ok(event) = res {
                match event.kind {
                    EventKind::Create(_) => {
                        let path = event.paths[0].clone();
                        let path = path.to_str().unwrap().trim_end_matches('~');

                        let root_folder = project_root::get_project_root().unwrap();
                        let relative_path = diff_paths(Path::new(path), root_folder).unwrap();

                        bash!("./tailwindcss -i ./assets/input.css -o ./dist/style.css");
                        println!("File changed: {:?}", relative_path);

                        sender.send(true).unwrap();
                        tera.write().await.full_reload().unwrap();
                    }
                    _ => {}
                }
            }
        }
    });
}