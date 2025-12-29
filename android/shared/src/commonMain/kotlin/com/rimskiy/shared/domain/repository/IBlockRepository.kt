package com.rimskiy.shared.domain.repository

import com.rimskiy.shared.data.model.Block
import com.rimskiy.shared.data.model.BlockWithBlockerInfo
import com.rimskiy.shared.data.model.CheckBlockResponse

interface IBlockRepository {
    suspend fun createBlock(blockedPlate: String, notifyOwner: Boolean = false, departureTime: String? = null): Result<Block>
    suspend fun getMyBlocks(): Result<List<Block>>
    suspend fun getBlocksForMyPlate(myPlate: String? = null): Result<List<BlockWithBlockerInfo>>
    suspend fun deleteBlock(blockId: String): Result<Unit>
    suspend fun checkBlock(plate: String): Result<CheckBlockResponse>
    suspend fun warnOwner(blockId: String): Result<Unit>
}

