package com.rimskiy.shared.di

import com.rimskiy.shared.data.api.ApiClient
import com.rimskiy.shared.data.local.SettingsManager
import com.rimskiy.shared.data.local.TokenManager
import com.rimskiy.shared.domain.repository.AuthRepository
import com.rimskiy.shared.domain.repository.BlockRepository
import com.rimskiy.shared.domain.repository.UserRepository
import com.rimskiy.shared.domain.usecase.*
import com.rimskiy.shared.platform.createSettings

class AppModule(
    private val baseUrl: String,
    private val ddnsUsername: String? = null,
    private val ddnsPassword: String? = null
) {
    // Single Responsibility: AppModule отвечает только за создание зависимостей
    private val settingsManager by lazy {
        SettingsManager(createSettings())
    }
    
    private val apiClient by lazy {
        ApiClient.create(baseUrl, ddnsUsername, ddnsPassword)
    }
    
    // Dependency Inversion: используем интерфейсы, а не конкретные реализации
    val authRepository: com.rimskiy.shared.domain.repository.IAuthRepository by lazy {
        AuthRepository(apiClient, settingsManager)
    }
    
    private val tokenManager by lazy { TokenManager(settingsManager) }
    
    val userRepository: com.rimskiy.shared.domain.repository.IUserRepository by lazy {
        UserRepository(apiClient, settingsManager, tokenManager)
    }
    
    val blockRepository: com.rimskiy.shared.domain.repository.IBlockRepository by lazy {
        BlockRepository(apiClient, settingsManager, tokenManager)
    }
    
    // Use Cases
    val startAuthUseCase: StartAuthUseCase by lazy {
        StartAuthUseCase(authRepository)
    }
    
    val verifyAuthUseCase: VerifyAuthUseCase by lazy {
        VerifyAuthUseCase(authRepository)
    }
    
    val logoutUseCase: LogoutUseCase by lazy {
        LogoutUseCase(authRepository)
    }
    
    val isAuthenticatedUseCase: IsAuthenticatedUseCase by lazy {
        IsAuthenticatedUseCase(authRepository)
    }
    
    val getProfileUseCase: GetProfileUseCase by lazy {
        GetProfileUseCase(userRepository)
    }
    
    val updateProfileUseCase: UpdateProfileUseCase by lazy {
        UpdateProfileUseCase(userRepository)
    }
    
    val getUserByPlateUseCase: GetUserByPlateUseCase by lazy {
        GetUserByPlateUseCase(userRepository)
    }
    
    val createBlockUseCase: CreateBlockUseCase by lazy {
        CreateBlockUseCase(blockRepository)
    }
    
    val getMyBlocksUseCase: GetMyBlocksUseCase by lazy {
        GetMyBlocksUseCase(blockRepository)
    }
    
    val getBlocksForMyPlateUseCase: GetBlocksForMyPlateUseCase by lazy {
        GetBlocksForMyPlateUseCase(blockRepository)
    }
    
    val deleteBlockUseCase: DeleteBlockUseCase by lazy {
        DeleteBlockUseCase(blockRepository)
    }

    val checkBlockUseCase: CheckBlockUseCase by lazy {
        CheckBlockUseCase(blockRepository)
    }

    val warnOwnerUseCase: WarnOwnerUseCase by lazy {
        WarnOwnerUseCase(blockRepository)
    }
    
    val recognizePlateUseCase: RecognizePlateUseCase by lazy {
        RecognizePlateUseCase(apiClient, settingsManager)
    }
    
    val getNotificationsUseCase: GetNotificationsUseCase by lazy {
        GetNotificationsUseCase(apiClient, settingsManager)
    }
    
    val markNotificationReadUseCase: MarkNotificationReadUseCase by lazy {
        MarkNotificationReadUseCase(apiClient, settingsManager)
    }
    
    val markAllNotificationsReadUseCase: MarkAllNotificationsReadUseCase by lazy {
        MarkAllNotificationsReadUseCase(apiClient, settingsManager)
    }
    
    val getUserPlatesUseCase: GetUserPlatesUseCase by lazy {
        GetUserPlatesUseCase(apiClient, settingsManager)
    }
    
    val createUserPlateUseCase: CreateUserPlateUseCase by lazy {
        CreateUserPlateUseCase(apiClient, settingsManager)
    }
    
    val deleteUserPlateUseCase: DeleteUserPlateUseCase by lazy {
        DeleteUserPlateUseCase(apiClient, settingsManager)
    }
    
    val setPrimaryPlateUseCase: SetPrimaryPlateUseCase by lazy {
        SetPrimaryPlateUseCase(apiClient, settingsManager)
    }
}

