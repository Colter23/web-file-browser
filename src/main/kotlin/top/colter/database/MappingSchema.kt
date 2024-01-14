package top.colter.database

import kotlinx.coroutines.Dispatchers
import kotlinx.datetime.Clock
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import org.jetbrains.exposed.sql.*
import org.jetbrains.exposed.sql.SqlExpressionBuilder.eq
import org.jetbrains.exposed.sql.kotlin.datetime.datetime
import org.jetbrains.exposed.sql.transactions.experimental.newSuspendedTransaction
import org.jetbrains.exposed.sql.transactions.transaction
import top.colter.database.MappingTable.folderPath
import top.colter.database.MappingTable.id
import top.colter.database.MappingTable.mountPath
import top.colter.database.MappingTable.order
import top.colter.database.MappingTable.remark
import top.colter.database.MappingTable.status
import top.colter.models.PathMapping


/**
 * 路径映射表
 */
object MappingTable : Table() {
    // ID
    val id = integer("id").autoIncrement()
    // 挂载路径
    val mountPath = varchar("mount_path", length = 255)
    // 本地文件路径
    val folderPath = varchar("folder_path", length = 255)
    // 备注
    val remark = varchar("remark", length = 255)
    // 排序
    val order = integer("order").default(0)

    // 状态 (0:正常 1:异常 2:停用 -1:删除)
    val status = integer("status").default(0)
    // 更新时间
    val updateTime = datetime("update_time")
    // 创建时间
    val createTime = datetime("create_time")

    override val primaryKey = PrimaryKey(id)
}


class MappingService(database: Database) {
    init {
        transaction(database) {
            SchemaUtils.create(MappingTable)
        }
    }

    suspend fun <T> dbQuery(block: suspend () -> T): T =
        newSuspendedTransaction(Dispatchers.IO) { block() }

    suspend fun create(mapping: PathMapping): Int = dbQuery {
        MappingTable.insert {
            it[mountPath] = mapping.mountPath
            it[folderPath] = mapping.folderPath.removeSuffix("/")
            it[remark] = mapping.remark ?: ""
            it[order] = mapping.order ?: 0
            it[status] = 0

            val now = Clock.System.now().toLocalDateTime(TimeZone.currentSystemDefault())
            it[updateTime] = now
            it[createTime] = now
        }[id]
    }

    suspend fun getAllMapping(): List<PathMapping> {
        return dbQuery {
            MappingTable.selectAll()
                .where { status neq -1 }
                .map { PathMapping(
                    id = it[id],
                    mountPath = it[mountPath],
                    folderPath = it[folderPath],
                    remark = it[remark],
                    order = it[order],
                )}
        }
    }

    suspend fun delete(id: Int) {
        dbQuery {
            MappingTable.deleteWhere { MappingTable.id.eq(id) }
        }
    }

}