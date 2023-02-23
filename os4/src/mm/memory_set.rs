
use super::{VirtPageNum, FrameTracker, VPNRange};


pub struct MapArea {
    vpn_range: VPNRange, 
    data_frame: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}

pub enum MapType {
    Identical,  //  恒等映射
    Framed,     // 新分配一个物理页帧与之对应
}
