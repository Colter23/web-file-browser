package top.colter.models

import kotlinx.serialization.Serializable

@Serializable
data class FileInfo (
    val name: String,
    val path: String,
    val modified: String,
    val size: Long,
    val extension: String
)
