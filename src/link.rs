use std::sync::Arc;
use std::fmt;

use Variable;

// Do not change this without updating the algorithms!
const BLOCK_SIZE: usize = 124;

const EMPTY: u64 = 0x0;
const BOOL: u64 = 0x1;
const F64: u64 = 0x2;
const STR: u64 = 0x3;

/// Stores link memory in chunks of 1024 bytes.
pub struct Block {
    data: [usize; BLOCK_SIZE],
    tys: [u64; 4]
}

impl Block {
    pub fn new() -> Block {
        Block {
            data: [0; BLOCK_SIZE],
            tys: [0; 4],
        }
    }

    pub fn var(&self, ind: u8) -> Variable {
        use std::mem::transmute;

        let k = ind as usize;
        assert!(k < BLOCK_SIZE);
        let i = k / 32;
        let j = k - i * 32;
        match self.tys[i] >> (j * 2) & 0x3 {
            EMPTY => panic!("Reading beyond end"),
            BOOL => Variable::bool(self.data[k] != 0),
            F64 => {
                Variable::f64(unsafe {
                    transmute::<usize, f64>(self.data[k])
                })
            }
            STR => {

                Variable::Text(unsafe {
                    transmute::<&usize, &Arc<String>>(&self.data[k])
                }.clone())
            }
            _ => panic!("Invalid type"),
        }
    }

    pub fn push(&mut self, var: &Variable, pos: usize) {
        use std::mem::transmute;

        let k = pos;
        assert!(k < BLOCK_SIZE);

        let i = k / 32;
        let j = k - i * 32;
        match *var {
            Variable::Bool(val, _) => {
                // Reset bits.
                self.tys[i] &= !(0x3 << (j * 2));
                // Sets new bits.
                self.tys[i] |= BOOL << (j * 2);
                self.data[k] = val as usize;
            }
            Variable::F64(val, _) => {
                // Reset bits.
                self.tys[i] &= !(0x3 << (j * 2));
                // Sets new bits.
                self.tys[i] |= F64 << (j * 2);
                self.data[k] = unsafe { transmute::<f64, usize>(val) };
            }
            Variable::Text(ref s) => {
                // Reset bits.
                self.tys[i] &= !(0x3 << (j * 2));
                // Sets new bits.
                self.tys[i] |= STR << (j * 2);
                self.data[k] = unsafe { transmute::<Arc<String>, usize>(s.clone()) };
            }
            _ => panic!("Expected `str`, `f64`, `bool`")
        }
    }
}

impl Clone for Block {
    fn clone(&self) -> Block {
        use std::mem::transmute;

        let mut data = self.data;
        for k in 0..BLOCK_SIZE {
            let i = k / 32;
            let j = k - i * 32;
            match self.tys[i] >> (j * 2) & 0x3 {
                EMPTY => break,
                STR => {
                    // Arc<String>
                    unsafe {
                        data[k] = transmute::<Arc<String>, usize>(
                            transmute::<&usize, &Arc<String>>(
                                &self.data[k]
                            ).clone());
                    }
                }
                _ => {}
            }
        }
        Block {
            data: data,
            tys: self.tys,
        }
    }
}

impl Drop for Block {
    fn drop(&mut self) {
        use std::mem::transmute;

        for k in 0..BLOCK_SIZE {
            let i = k / 32;
            let j = k - i * 32;
            match self.tys[i] >> (j * 2) & 0x3 {
                EMPTY => break,
                STR => {
                    // Arc<String>
                    unsafe {
                        drop(transmute::<usize, Arc<String>>(self.data[k]))
                    }
                }
                _ => {}
            }
        }
    }
}

impl fmt::Debug for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Block")
    }
}

#[derive(Debug, Clone)]
pub struct Slice {
    pub block: Arc<Block>,
    pub start: u8,
    pub end: u8,
}

impl Slice {
    pub fn new() -> Slice {
        Slice {
            block: Arc::new(Block::new()),
            start: 0,
            end: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Link {
    pub slices: Vec<Slice>,
}

impl Link {
    pub fn new() -> Link {
        Link {
            slices: vec![]
        }
    }

    pub fn head(&self) -> Option<Box<Variable>> {
        if self.slices.len() == 0 { None }
        else {
            let first = &self.slices[0];
            if first.start < first.end {
                Some(Box::new(first.block.var(first.start)))
            } else {
                None
            }
        }
    }

    pub fn tail(&self) -> Link {
        if self.slices.len() == 0 { Link::new() }
        else {
            let first = &self.slices[0];
            let mut l = Link::new();
            if first.start < first.end {
                if first.start + 1 < first.end {
                    l.slices.push(first.clone());
                    l.slices[0].start += 1;
                }
            }
            for slice in self.slices.iter().skip(1) {
                l.slices.push(slice.clone())
            }
            l
        }
    }

    pub fn is_empty(&self) -> bool { self.slices.len() == 0 }

    pub fn add(&self, other: &Link) -> Link {
        let mut slices = Vec::with_capacity(self.slices.len() + other.slices.len());
        slices.extend_from_slice(&self.slices);
        slices.extend_from_slice(&other.slices);
        Link {
            slices: slices
        }
    }

    pub fn push(&mut self, v: &Variable) -> Result<(), String> {
        match v {
            &Variable::Bool(_, _) |
            &Variable::F64(_, _) |
            &Variable::Text(_) => {
                if self.slices.len() > 0 {
                    let mut last = self.slices.last_mut().unwrap();
                    if (last.end as usize) < BLOCK_SIZE {
                        Arc::make_mut(&mut last.block).push(v, last.end as usize);
                        last.end += 1;
                        return Ok(());
                    }
                }

                self.slices.push(Slice::new());
                let mut last = self.slices.last_mut().unwrap();
                Arc::make_mut(&mut last.block).push(v, 0);
                last.end = 1;
                Ok(())
            }
            &Variable::Link(ref link) => {
                for slice in &link.slices {
                    for i in slice.start..slice.end {
                        try!(self.push(&slice.block.var(i)))
                    }
                }
                Ok(())
            }
            _ => return Err("Expected `bool`, `f64` or `str`".into())
        }
    }
}
