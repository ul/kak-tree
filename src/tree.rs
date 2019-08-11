use tree_sitter::{Node, Range};

pub fn named_children<'a>(node: &'a Node) -> impl Iterator<Item = Node<'a>> {
    (0..node.child_count()).map(move |i| node.child(i).unwrap())
}

pub fn shrink_to_range<'a>(root_node: Node<'a>, range: &Range) -> Node<'a> {
    let mut node = root_node;
    'outer: loop {
        let parent = node;
        for child in parent.children() {
            if child.range().start_byte <= range.start_byte
                && range.end_byte <= child.range().end_byte
            {
                node = child;
                continue 'outer;
            }
        }
        return highest_node_of_same_range(node);
    }
}

pub fn nodes_in_range<'a>(root_node: Node<'a>, range: &Range) -> Vec<Node<'a>> {
    let mut nodes = Vec::new();
    let node = shrink_to_range(root_node, range);
    if node.range().start_byte >= range.start_byte && range.end_byte >= node.range().end_byte {
        nodes.push(node);
        return nodes;
    }
    for child in node.children() {
        if child.range().start_byte <= range.start_byte && range.end_byte >= child.range().end_byte
        {
            nodes.extend(nodes_in_range(
                child,
                &Range {
                    start_byte: range.start_byte,
                    start_point: range.start_point,
                    end_byte: child.range().end_byte,
                    end_point: child.range().end_point,
                },
            ));
        } else if child.range().start_byte >= range.start_byte
            && range.end_byte <= child.range().end_byte
        {
            nodes.extend(nodes_in_range(
                child,
                &Range {
                    start_byte: child.range().start_byte,
                    start_point: child.range().start_point,
                    end_byte: range.end_byte,
                    end_point: range.end_point,
                },
            ));
        } else if child.range().start_byte >= range.start_byte
            && range.end_byte >= child.range().end_byte
        {
            nodes.push(child);
        }
    }
    nodes
}

fn highest_node_of_same_range<'a>(current_node: Node<'a>) -> Node<'a> {
    let start_byte = current_node.start_byte();
    let end_byte = current_node.end_byte();
    let mut node = current_node;
    while let Some(parent) = node.parent().and_then(|parent| {
        if parent.start_byte() < start_byte || end_byte < parent.end_byte() {
            None
        } else {
            Some(parent)
        }
    }) {
        node = parent
    }
    node
}
