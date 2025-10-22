use std::env;
use rayon::prelude::*;
use std::path::PathBuf;
use swh_graph::graph::*;
use swh_graph::NodeType;
use swh_graph_stdlib::*;
use swh_graph_stdlib::FsTree::Directory;
use std::result::Result::Ok;
use anyhow::*;
use std::collections::HashMap;
use std::*;
use std::path::Path;
use serde_json;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let gpath = &args[1];

    let graph = swh_graph::graph::load_full::<swh_graph::mph::DynMphf>(PathBuf::from(gpath))
	.expect("Could not load graph");
    let props = graph.properties();

    let origins: Vec<usize> = (0..graph.num_nodes())
        .into_iter()
	.filter(|&node| props.node_type(node) == NodeType::Origin)
	.filter(|&node| {
            if let Some(bytes) = props.message(node) {
		if let Ok(url) = str::from_utf8(&bytes) {
                    url.starts_with("https://github.com")
		} else {
                    false
		}
            } else {
		false
            }
	})
	.collect();

    let mut data: HashMap<String, HashMap<String, Vec<String>>> = HashMap::new();

    for ori in origins {
	if let Ok(Some(snp)) = swh_graph_stdlib::find_latest_snp(&graph, ori) {
	    if let Ok(Some(hd)) = swh_graph_stdlib::find_head_rev(&graph, snp.0) {
		if let Ok(Some(rt)) = swh_graph_stdlib::find_root_dir(&graph, hd) {
		    if let Ok(Some(ghw)) = swh_graph_stdlib::fs_resolve_path(&graph, rt, ".github/workflows") {
			let url = str::from_utf8(&props.message(ori).unwrap()).unwrap().to_string();
			let tree = fs_ls_tree(&graph, ghw).unwrap();
			let mut ls: Vec<String> = Vec::new();
			match tree {
			    Directory(dir) => {
				for k in dir.keys() {
				    ls.push(str::from_utf8(&k).unwrap().to_string());
				    println!("{:?}", k.read());
				}
			    },
			    _ => bail!(""),
			}
			let mut h: HashMap<String, Vec<String>> = HashMap::new();
			h.insert("./github/workflow".to_string(), ls);
			data.insert(url.to_string(), h);
		    }
		}
	    }
	}
    }

    let path = Path::new("projects.json");
    fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}
