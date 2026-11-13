//! Beyond the Book: the other standard collections, and when to use each.
//! Run: cargo run -p collections --example more_collections

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashSet, VecDeque};

fn main() {
    // VecDeque: a queue you can push and pop at BOTH ends cheaply.
    let mut queue = VecDeque::from(["Ama", "Kofi"]);
    queue.push_back("Esi"); // join at the back
    queue.push_front("VIP"); // jump the queue
    println!("next: {:?}, queue now {queue:?}", queue.pop_front());

    // HashSet: unique values, fast "is it in there?".
    let mut visited = HashSet::new();
    for city in ["Accra", "Kumasi", "Accra", "Tamale"] {
        if !visited.insert(city) {
            println!("already visited {city}");
        }
    }
    let planned: HashSet<_> = ["Tamale", "Cape Coast"].into_iter().collect();
    let mut both: Vec<_> = visited.intersection(&planned).collect();
    both.sort();
    println!("visited and planned: {both:?}");

    // BTreeMap / BTreeSet: like HashMap / HashSet, but kept SORTED by key.
    let mut grades = BTreeMap::new();
    grades.insert("Kofi", 72);
    grades.insert("Ama", 91);
    grades.insert("Esi", 85);
    println!("sorted by name: {grades:?}");
    let names_from_e: Vec<_> = grades.range("E"..).map(|(name, _)| *name).collect();
    println!("names from E: {names_from_e:?}");
    let unique_sorted: BTreeSet<_> = [5, 1, 3, 1, 5].into_iter().collect();
    println!("BTreeSet: {unique_sorted:?}");

    // BinaryHeap: always hands back the LARGEST item first (a priority queue).
    let mut triage = BinaryHeap::new();
    triage.push((2, "sprained ankle"));
    triage.push((9, "broken arm"));
    triage.push((5, "fever"));
    while let Some((urgency, patient)) = triage.pop() {
        println!("treat {patient} (urgency {urgency})");
    }
    // Wrap in Reverse for smallest-first.
    let mut smallest_first: BinaryHeap<_> = [3, 1, 2].into_iter().map(Reverse).collect();
    println!("smallest: {:?}", smallest_first.pop().map(|Reverse(n)| n));
}
