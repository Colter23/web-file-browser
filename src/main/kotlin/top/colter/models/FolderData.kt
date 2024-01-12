package top.colter.models

import kotlinx.serialization.Serializable

@Serializable
data class FolderData (
    val path: String,
    val folder: List<FolderInfo>,
    val file: List<FileInfo>,
)