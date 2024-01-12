package top.colter.models

import kotlinx.serialization.Serializable

@Serializable
data class FolderInfo (
    val name: String,
    val path: String,
    val modified: String
)
