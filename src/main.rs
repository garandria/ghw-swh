use std::env;
use rayon::prelude::*;
use std::path::PathBuf;
use swh_graph::graph::*;
use swh_graph::NodeType;
use swh_graph_stdlib;
use std::result::Result::Ok;
use anyhow::*;


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
		if let Ok(url) = std::str::from_utf8(&bytes) {
                    url.starts_with("https://github.com")
		} else {
                    false
		}
            } else {
		false
            }
	})
	.collect();

    origins.into_par_iter().for_each(|ori| {
	if let Ok(Some(snp)) = swh_graph_stdlib::find_latest_snp(&graph, ori) {
	    if let Ok(Some(hd)) = swh_graph_stdlib::find_head_rev(&graph, snp.0) {
		if let Ok(Some(rt)) = swh_graph_stdlib::find_root_dir(&graph, hd) {
		    if let Ok(Some(_)) = swh_graph_stdlib::fs_resolve_path(&graph, rt, ".github/workflows") {
			println!("{}", std::str::from_utf8(&props.message(ori).unwrap()).unwrap());
		    }
		}
	    }
	}
    });
    Ok(())
}
