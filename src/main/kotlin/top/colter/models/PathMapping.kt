package top.colter.models

import kotlinx.serialization.Serializable


@Serializable
data class PathMapping(
    // ID
    val id: Int? = null,
    // 挂载路径
    val mountPath: String,
    // 本地文件路径
    val folderPath: String,
    // 备注
    val remark: String? = "",
    // 排序
    val order: Int? = 0
)