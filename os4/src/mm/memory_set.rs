
use super::{VirtPageNum, FrameTracker, VPNRange};


pub struct MapArea {
    vpn_range: VPNRange，
    data_frame: BTreeMap<VirtPageNum, FrameTracker>,
    map_type: MapType,
    map_perm: MapPermission,
}