package top.colter.models

import kotlinx.datetime.LocalDateTime
import kotlinx.serialization.Serializable


@Serializable
sealed interface Folder {
    val name: String
    val path: String
}

@Serializable
data class VirtualFolder(
    override val name: String,
    override val path: String,
    val children: MutableList<Folder> = mutableListOf()
): Folder


@Serializable
data class RealFolder(
    override val name: String,
    override val path: String,
    val realPath: String,
): Folder