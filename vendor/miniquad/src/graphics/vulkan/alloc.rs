//! Minimal device-memory suballocator: 64 MiB blocks per memory type, first-fit free list.
//! Keeps the number of vkAllocateMemory calls low (the PS5's limit is unknown; Vulkan only
//! guarantees 4096 allocations).
use ash::vk;

const BLOCK: u64 = 64 << 20;

pub struct Allocation {
    pub memory: vk::DeviceMemory,
    pub offset: u64,
    pub size: u64,
    pub ptr: *mut u8, // null unless host visible
    block: usize,
}

struct Block {
    memory: vk::DeviceMemory,
    type_index: u32,
    size: u64,
    ptr: *mut u8,
    free: Vec<(u64, u64)>, // (offset, size), sorted by offset
}

pub struct Allocator {
    props: vk::PhysicalDeviceMemoryProperties,
    blocks: Vec<Option<Block>>,
}

impl Allocator {
    pub fn new(props: vk::PhysicalDeviceMemoryProperties) -> Self {
        Allocator { props, blocks: vec![] }
    }

    pub fn find_type(&self, bits: u32, want: vk::MemoryPropertyFlags) -> Option<u32> {
        (0..self.props.memory_type_count).find(|&i| {
            bits & (1 << i) != 0 && self.props.memory_types[i as usize].property_flags.contains(want)
        })
    }

    pub unsafe fn alloc(&mut self, device: &ash::Device, req: vk::MemoryRequirements, host: bool) -> Allocation {
        let want = if host {
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
        } else {
            vk::MemoryPropertyFlags::DEVICE_LOCAL
        };
        let type_index = self
            .find_type(req.memory_type_bits, want)
            .or_else(|| self.find_type(req.memory_type_bits, vk::MemoryPropertyFlags::empty()))
            .expect("no Vulkan memory type");
        let align = req.alignment.max(256);
        for (bi, b) in self.blocks.iter_mut().enumerate() {
            let Some(b) = b else { continue };
            if b.type_index != type_index { continue; }
            if let Some(a) = Self::take(b, req.size, align) {
                return Allocation { memory: b.memory, offset: a, size: req.size, ptr: if b.ptr.is_null() { b.ptr } else { b.ptr.add(a as usize) }, block: bi };
            }
        }
        let size = BLOCK.max(req.size);
        let memory = device
            .allocate_memory(&vk::MemoryAllocateInfo::default().allocation_size(size).memory_type_index(type_index), None)
            .expect("vkAllocateMemory");
        let host_visible = self.props.memory_types[type_index as usize].property_flags.contains(vk::MemoryPropertyFlags::HOST_VISIBLE);
        let ptr = if host_visible {
            device.map_memory(memory, 0, vk::WHOLE_SIZE, vk::MemoryMapFlags::empty()).expect("map") as *mut u8
        } else {
            std::ptr::null_mut()
        };
        let mut b = Block { memory, type_index, size, ptr, free: vec![(0, size)] };
        let a = Self::take(&mut b, req.size, align).unwrap();
        let bi = match self.blocks.iter().position(|b| b.is_none()) {
            Some(i) => { self.blocks[i] = Some(b); i }
            None => { self.blocks.push(Some(b)); self.blocks.len() - 1 }
        };
        let blk = self.blocks[bi].as_ref().unwrap();
        Allocation { memory, offset: a, size: req.size, ptr: if blk.ptr.is_null() { blk.ptr } else { blk.ptr.add(a as usize) }, block: bi }
    }

    fn take(b: &mut Block, size: u64, align: u64) -> Option<u64> {
        for i in 0..b.free.len() {
            let (off, len) = b.free[i];
            let start = (off + align - 1) / align * align;
            if start + size <= off + len {
                b.free.remove(i);
                if start > off { b.free.insert(i, (off, start - off)); }
                let end = start + size;
                if end < off + len {
                    let pos = b.free.iter().position(|f| f.0 > end).unwrap_or(b.free.len());
                    b.free.insert(pos, (end, off + len - end));
                }
                return Some(start);
            }
        }
        None
    }

    pub unsafe fn free(&mut self, device: &ash::Device, a: Allocation) {
        let Some(b) = self.blocks[a.block].as_mut() else { return };
        let pos = b.free.iter().position(|f| f.0 > a.offset).unwrap_or(b.free.len());
        b.free.insert(pos, (a.offset, a.size));
        // merge neighbours
        let mut merged: Vec<(u64, u64)> = Vec::with_capacity(b.free.len());
        for &(o, l) in &b.free {
            if let Some(last) = merged.last_mut() {
                if last.0 + last.1 >= o { last.1 = (o + l).max(last.0 + last.1) - last.0; continue; }
            }
            merged.push((o, l));
        }
        b.free = merged;
        let (empty, ty) = (b.free.len() == 1 && b.free[0] == (0, b.size), b.type_index);
        if empty && self.blocks.iter().flatten().filter(|x| x.type_index == ty).count() > 1 {
            let b = self.blocks[a.block].take().unwrap();
            device.free_memory(b.memory, None);
        }
    }
}
