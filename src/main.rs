use std::env;
use std::path::PathBuf;
use swh_graph::graph::*;
use swh_graph::NodeType;
use swh_graph_stdlib;

fn main() {
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

    for ori in origins {
	let snp = match swh_graph_stdlib::find_latest_snp(&graph, ori) {
	    Ok(s) => s.unwrap(),
	    Err(_) => continue,
	};
	let hd = match swh_graph_stdlib::find_head_rev(&graph, snp.0) {
	    Ok(h) => match h {
		Some(h_) => h_,
		None => continue,
	    },
	    Err(_) => continue,
	};
	let rt = match swh_graph_stdlib::find_root_dir(&graph, hd) {
	    Ok(r) => r.unwrap(),
	    Err(_) => continue,
	};
	let cnt = match swh_graph_stdlib::fs_resolve_path(&graph, rt, ".github/workflows") {
	    Ok(c) => match c {
		Some (c_) => println!("{}", std::str::from_utf8(&props.message(ori).unwrap()).unwrap()),
		None => continue,
	    },
	    Err(_) => continue,
	};
    }
}
