use std::collections::{VecDeque, HashMap};
use std::env;
//file IO
use std::path::Path;
use std::fs::File;
use std::io::BufRead;
use std::io;
//mult threading
use std::thread;
use std::sync::{Arc, Mutex, mpsc};
//timeing
use std::time::Instant;

//takes from the line read rust by example
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

struct Graph {
    edge: HashMap<i32, Vec<i32>>,
}
impl Graph {
    fn new(size:usize) -> Self {
        Self {
            edge: HashMap::with_capacity(size),
        }
    }
    fn add_edge(&mut self, i:&i32, j:&i32) {
        //needs to be in two differnt blocks because borrowing
        {
            let iedges = self.edge.get_mut(i);
            if let Some(vec) = iedges {
                vec.push(*j);
            } else {
                self.edge.insert(*i, Vec::new());
            }

        }
        {
            let jedges = self.edge.get_mut(j);
            if let Some(vec) = jedges {
                vec.push(*i);
            } else {
                self.edge.insert(*j, Vec::new());
            }

        }

    }
    fn read_from(filename:&str, size:usize, delim:String) -> Self {
        let mut g = Graph::new(size);
        //open file
        if let Ok(lines) = read_lines(filename) {
            for line in lines {
                if let Ok(s) = line {
                    if !s.starts_with('#') {
                        let mut iter = s.split(&delim);
                        let s1 = iter.next().unwrap();
                        let s2 = iter.next().unwrap();
                        g.add_edge(&s1.parse::<i32>().unwrap(),
                                   &s2.parse::<i32>().unwrap());
                    }
                }
            }
        } if let Err(lines) = read_lines(filename) {
            eprintln!("file couldn't open: {}", lines);
        }
        g 
    }
    fn bfs(&self, r:i32) {
        //que of all nodes that need to be vististed
        let mut verts = VecDeque::new();
        //boolean array to check if node has been visted before
        let mut visted = vec!(false; self.edge.capacity());
        verts.push_back(r);
        while verts.len() != 0 {
            //if the que not empty
            if let Some(n) = verts.pop_front() {
                if let Some(edges) = self.edge.get(&n) {
                    for v in edges {
                        let vsize = (*v) as usize; //panic if too big
                        if !visted[vsize] {
                            //println!("{}", *v);
                            visted[vsize] = true;
                            verts.push_back(*v);
                        }
                    }
                }
            }
            //println!("{:?}", verts)
        }
    }
    fn parra_bfs(&self, r:i32) {
        let edges = Arc::new(self.edge.clone()); //is now immutable
        //boolean array to check if node has been visted before
        let visted = Arc::new(Mutex::new(vec!(false; self.edge.capacity())));
        //println!("{:?}", edges);
        //que of all nodes that need to be vististed. 
        let mut verts: VecDeque<i32> = VecDeque::new();
        verts.push_back(r);
        let mut vist = visted.lock().unwrap();
        vist[r as usize] = true;//mark root as found
        drop(vist);
        loop {
            //if there is no nodes left
            if verts.is_empty() {
                break;
            }

            let (tx, rx) = mpsc::channel(); //collects the new visted nodes 
            let mut handles = vec![]; 
            for v in verts {
                let v = Arc::new(v); //must be sync so no handging pointers
                //allows mult owners
                let edges = Arc::clone(&edges);
                let visted = Arc::clone(&visted);
                //allows mult tranmistions
                let tx = tx.clone();
                let handle = thread::spawn(move || {      
                    if let Some(edges) = edges.get(&v) {
                        //println!("{}", edges[*v][i]);
                        for e in edges {
                            let mut vist = visted.lock().unwrap();
                            let es = (*e) as usize; //panic if too big
                            if !vist[es] {
                                vist[es] = true;
                                let _= tx.send(e.clone()); 
                            }
                        }
                    }
                });
                handles.push(handle);
            }
             
            for handle in handles {
                handle.join().unwrap();
            }
           //println!("{}", rec);
            verts = VecDeque::new(); //allows to be borrowed again
            drop(tx); //closing the channel
            for rec in rx {
                //println!("{}", rec);
                if !verts.contains(&rec) {
                    verts.push_back(rec);
                }
            }
        
        }
    }
}
fn main() {
    let args: Vec<String> =  env::args().collect();
    let mut g = Graph::new(0);
    g.add_edge(&1,&0);
    g.add_edge(&2,&0);
    g.add_edge(&3,&0);
    g.add_edge(&3,&4);
    g.add_edge(&3,&5);
    //g.bfs(0);

    println!("{:?}", args);
    let big_g = Graph::read_from(&args[1], args[1].parse::<usize>().unwrap(), args[2].clone());
    //println!("{:?}", big_g.edge);   
    let now = Instant::now();
    big_g.parra_bfs(254913);
    let end = now.elapsed();
    println!("parrallel {:.2?}", end);

    let now = Instant::now();
    big_g.bfs(254913);
    let end = now.elapsed();
    println!("sequential {:.2?}", end);


}
