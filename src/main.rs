use rand::Rng;

// Define the maximum number of levels the Skip List can have.
const MAX_LEVEL: usize = 16;
// Probability of promoting a node to the next level (p=0.5)
const P: f64 = 0.5;

// A single node in the Skip List.
// It uses raw pointers for the `next` array, which is typical for
// performance-critical, linked-style structures in Rust when
// managed pointers (like `Box` or `Rc`) lead to complexity or overhead.
// We use `unsafe` blocks appropriately for pointer manipulation.
struct Node {
    value: i32,
    // An array of raw pointers, one for each level.
    next: [Option<*mut Node>; MAX_LEVEL],
}

impl Node {
    // Creates a new node with a given value and specific level count.
    fn new(value: i32, _level: usize) -> Node {
        // Initialise all `next` pointers up to `level` to `None`.
        Node {
            value,
            next: [None; MAX_LEVEL],
        }
    }
}

// The main Skip List structure.
pub struct SkipList {
    head: Option<*mut Node>, // A pointer to the sentinel head node (no data).
    level: usize,            // Current maximum level of the entire list.
}

impl SkipList {
    pub fn new() -> SkipList {
        // Create the sentinel head node. It has a dummy value (-1) and MAX_LEVEL next pointers.
        let head_node = Box::into_raw(Box::new(Node::new(-1, MAX_LEVEL)));
        SkipList {
            head: Some(head_node),
            level: 0,
        }
    }

    /// Inserts a value into the Skip List.
    pub fn insert(&mut self, value: i32) {
        let mut update: [Option<*mut Node>; MAX_LEVEL] = [None; MAX_LEVEL];
        let mut current = self.head;

        // 1. Find the insertion point at all levels
        for i in (0..=self.level).rev() {
            unsafe {
                while let Some(node_ptr) = current {
                    let node = &mut *node_ptr;
                    // Move right if the next node exists and its value is less than the new value.
                    if node.next[i].is_some() && (*node.next[i].unwrap()).value < value {
                        current = node.next[i];
                    } else {
                        break;
                    }
                }
                // Record the predecessor at this level (where insertion happens)
                update[i] = current;
            }
        }
        
        // At this point, `current` is the node before the insertion point on level 0.
        // We ensure we are at level 0 by getting the first `next` of the final `current`.
        current = Some(update[0].unwrap());

        // Check for duplicates (optional, for a Set-like list)
        unsafe {
            if current.is_some() {
                let node = &*current.unwrap();
                if node.next[0].is_some() && (*node.next[0].unwrap()).value == value {
                    // Value already exists, do nothing
                    return;
                }
            }
        }

        // 2. Determine the new node's level
        let new_level = Self::random_level();

        // 3. Update the list's max level if necessary
        if new_level > self.level {
            // Update the `update` array for the new levels to point to the head node
            for i in (self.level + 1)..=new_level {
                update[i] = self.head;
            }
            self.level = new_level;
        }

        // 4. Create and link the new node
        let new_node_ptr = Box::into_raw(Box::new(Node::new(value, new_level + 1)));

        // Link the new node into the list from level 0 up to `new_level`
        for i in 0..=new_level {
            unsafe {
                let predecessor_ptr = update[i].unwrap();
                let predecessor = &mut *predecessor_ptr;

                // New node's next pointer points to the old successor
                (*new_node_ptr).next[i] = predecessor.next[i];
                // Predecessor's next pointer points to the new node
                predecessor.next[i] = Some(new_node_ptr);
            }
        }
    }

    /// Searches for a value in the Skip List. Returns true if found.
    pub fn search(&self, value: i32) -> bool {
        let mut current = self.head;

        // Start from the highest current level and work down
        for i in (0..=self.level).rev() {
            unsafe {
                while let Some(node_ptr) = current {
                    let node = &*node_ptr;
                    // Move right if the next node exists and its value is less than the search value.
                    if node.next[i].is_some() && (*node.next[i].unwrap()).value < value {
                        current = node.next[i];
                    } else {
                        break; // Drop down to the next level
                    }
                }
            }
        }
        
        // After the loops, `current` should be the node before the potential match on level 0.
        // We now check the node immediately after `current` on level 0.
        unsafe {
            if let Some(node_ptr) = current {
                let node = &*node_ptr;
                if let Some(next_ptr) = node.next[0] {
                    // Check if the value of the next node matches
                    return (*next_ptr).value == value;
                }
            }
        }
        false
    }

    /// Generates a random level for a new node.
    /// The probability of increasing the level is P (0.5).
    fn random_level() -> usize {
        let mut lvl = 0;
        let mut rng = rand::rng();

        // Keep incrementing the level as long as a random number
        // is less than the probability P, up to MAX_LEVEL.
        while rng.random::<f64>() < P && lvl < MAX_LEVEL - 1 {
            lvl += 1;
        }
        lvl
    }
}
    
impl Drop for SkipList {
    fn drop(&mut self) {
        let mut current = self.head;
        // Traverse only the base list (level 0) to free all nodes.
        while let Some(node_ptr) = current {
            unsafe {
                // Get the next pointer at level 0 before deallocating the current node.
                let next = (*node_ptr).next[0];

                // Take ownership of the raw pointer and drop the Box, which deallocates the Node.
                let _ = Box::from_raw(node_ptr);
                current = next;
            }
        }
    }
}

fn main() {
    let mut skip_list = SkipList::new();
    let values = vec![3, 6, 9, 2, 11, 1, 4];
    
    println!("Inserting values: {:?}", values);
    for &val in &values {
        skip_list.insert(val);
    }

    println!("\n--- Search Results ---");
    let search_values = vec![4, 5, 11, 0];
    for &val in &search_values {
        println!("Search for {}: {}", val, skip_list.search(val));
    }
    // Output should be: true, false, true, false
}