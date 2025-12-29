package com.rimskiy.shared.domain.usecase

import com.rimskiy.shared.data.model.Block
import com.rimskiy.shared.data.model.BlockWithBlockerInfo
import com.rimskiy.shared.data.model.CheckBlockResponse
import com.rimskiy.shared.domain.repository.IBlockRepository

class CreateBlockUseCase(
    private val blockRepository: IBlockRepository
) {
    suspend operator fun invoke(blockedPlate: String, notifyOwner: Boolean = false, departureTime: String? = null): Result<Block> {
        return blockRepository.createBlock(blockedPlate, notifyOwner, departureTime)
    }
}

class GetMyBlocksUseCase(
    private val blockRepository: IBlockRepository
) {
    suspend operator fun invoke(): Result<List<Block>> {
        return blockRepository.getMyBlocks()
    }
}

class GetBlocksForMyPlateUseCase(
    private val blockRepository: IBlockRepository
) {
    suspend operator fun invoke(myPlate: String? = null): Result<List<BlockWithBlockerInfo>> {
        return blockRepository.getBlocksForMyPlate(myPlate)
    }
}

class WarnOwnerUseCase(
    private val blockRepository: IBlockRepository
) {
    suspend operator fun invoke(blockId: String): Result<Unit> {
        return blockRepository.warnOwner(blockId)
    }
}

class DeleteBlockUseCase(
    private val blockRepository: IBlockRepository
) {
    suspend operator fun invoke(blockId: String): Result<Unit> {
        return blockRepository.deleteBlock(blockId)
    }
}

class CheckBlockUseCase(
    private val blockRepository: IBlockRepository
) {
    suspend operator fun invoke(plate: String): Result<CheckBlockResponse> {
        return blockRepository.checkBlock(plate)
    }
}

