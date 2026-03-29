use graphqlite::Graph;

type AppError = Box<dyn std::error::Error>;

pub fn run() -> Result<(), AppError> {
    let g = Graph::open(":memory:")?;

    // Add nodes
    g.upsert_node("alice", [("name", "Alice"), ("age", "30")], "Person")?;
    g.upsert_node("bob", [("name", "Bob"), ("age", "25")], "Person")?;

    // Add edge
    g.upsert_edge("alice", "bob", [("since", "2020")], "KNOWS")?;

    // Query
    println!("{:?}", g.stats()?);           // GraphStats { nodes: 2, edges: 1 }
    println!("{:?}", g.get_neighbors("alice")?);

    // Graph algorithms
    let ranks = g.pagerank(0.85, 20)?;
    let communities = g.community_detection(10)?;

    dbg!(ranks);
    dbg!(communities);

    Ok(())
}
