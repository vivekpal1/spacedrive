
export type ClientCommand = { key: "LocScanFull", params: { location_id: bigint, } } | { key: "FileScanQuick", params: { file_id: bigint, } } | { key: "FileScanFull", params: { file_id: bigint, } } | { key: "FileDelete", params: { file_id: bigint, } } | { key: "TagCreate", params: { name: string, color: string, } } | { key: "TagAssign", params: { file_id: bigint, tag_id: bigint, } } | { key: "TagDelete", params: { tag_id: bigint, } } | { key: "LocDelete", params: { location_id: bigint, } } | { key: "LibDelete", params: { library_id: bigint, } } | { key: "SysVolumeUnmount", params: { volume_id: bigint, } };