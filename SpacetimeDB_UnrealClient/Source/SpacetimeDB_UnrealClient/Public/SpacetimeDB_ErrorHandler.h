// Copyright SpacetimeDB. All Rights Reserved.

#pragma once

#include "CoreMinimal.h"
#include "SpacetimeDB_UnrealClient.h"

/**
 * Enum defining severity levels for SpacetimeDB errors
 */
UENUM(BlueprintType)
enum class ESpacetimeDBErrorSeverity : uint8
{
    // Informational message, not an error
    Info = 0,
    
    // Warning that doesn't prevent operation but should be addressed
    Warning = 1,
    
    // Error that prevents a specific operation but doesn't break the connection
    Error = 2,
    
    // Critical error that affects the connection or general functionality
    Critical = 3,
    
    // Fatal error that requires immediate attention and might crash the application
    Fatal = 4
};

/**
 * Struct containing detailed error information from SpacetimeDB
 */
USTRUCT(BlueprintType)
struct SPACETIMEDB_UNREALCLIENT_API FSpacetimeDBErrorInfo
{
    GENERATED_BODY()
    
    /** Error message */
    UPROPERTY(BlueprintReadOnly, Category = "SpacetimeDB|Error")
    FString Message;
    
    /** Error category or source */
    UPROPERTY(BlueprintReadOnly, Category = "SpacetimeDB|Error")
    FString Category;
    
    /** Error code if available (0 if not applicable) */
    UPROPERTY(BlueprintReadOnly, Category = "SpacetimeDB|Error")
    int32 Code = 0;
    
    /** Error severity level */
    UPROPERTY(BlueprintReadOnly, Category = "SpacetimeDB|Error")
    ESpacetimeDBErrorSeverity Severity = ESpacetimeDBErrorSeverity::Error;
    
    /** Additional context or details about the error */
    UPROPERTY(BlueprintReadOnly, Category = "SpacetimeDB|Error")
    FString Context;
    
    /** Whether this error was recovered from automatically */
    UPROPERTY(BlueprintReadOnly, Category = "SpacetimeDB|Error")
    bool bAutoRecovered = false;
    
    /** Default constructor */
    FSpacetimeDBErrorInfo() {}
    
    /** Constructor with basic error information */
    FSpacetimeDBErrorInfo(
        const FString& InMessage,
        ESpacetimeDBErrorSeverity InSeverity = ESpacetimeDBErrorSeverity::Error,
        const FString& InCategory = TEXT("General"),
        int32 InCode = 0,
        const FString& InContext = TEXT(""),
        bool bInAutoRecovered = false
    )
        : Message(InMessage)
        , Category(InCategory)
        , Code(InCode)
        , Severity(InSeverity)
        , Context(InContext)
        , bAutoRecovered(bInAutoRecovered)
    {
    }
};

/**
 * Static utility class for handling and logging SpacetimeDB errors
 */
class SPACETIMEDB_UNREALCLIENT_API FSpacetimeDBErrorHandler
{
public:
    /**
     * Logs an error and returns an error info struct
     * 
     * @param Message Error message
     * @param Severity Error severity
     * @param Category Error category
     * @param Code Error code
     * @param Context Additional context
     * @param bAutoRecovered Whether the error was automatically recovered from
     * @return Error info struct that can be passed to delegates
     */
    static FSpacetimeDBErrorInfo LogError(
        const FString& Message,
        ESpacetimeDBErrorSeverity Severity = ESpacetimeDBErrorSeverity::Error,
        const FString& Category = TEXT("General"),
        int32 Code = 0,
        const FString& Context = TEXT(""),
        bool bAutoRecovered = false
    );
    
    /**
     * Handles an error from an FFI call and logs it appropriately
     * 
     * @param FunctionName Name of the FFI function that was called
     * @param ErrorMessage Error message from the FFI call
     * @param bLogStackTrace Whether to include a stack trace in the log
     * @return Error info struct that can be passed to delegates
     */
    static FSpacetimeDBErrorInfo HandleFFIError(
        const FString& FunctionName,
        const FString& ErrorMessage,
        bool bLogStackTrace = true
    );
    
    /**
     * Parses an error message from FFI and extracts detailed information
     * 
     * @param ErrorMessage Error message from FFI
     * @param OutInfo Struct to populate with parsed information
     * @return True if parsing was successful
     */
    static bool ParseFFIErrorMessage(const FString& ErrorMessage, FSpacetimeDBErrorInfo& OutInfo);
}; 