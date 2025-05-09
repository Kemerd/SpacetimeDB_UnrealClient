// Copyright SpacetimeDB. All Rights Reserved.

#include "SpacetimeDB_ErrorHandler.h"
#include "Misc/StackTrace.h"
#include "Misc/Parse.h"

FSpacetimeDBErrorInfo FSpacetimeDBErrorHandler::LogError(
    const FString& Message,
    ESpacetimeDBErrorSeverity Severity,
    const FString& Category,
    int32 Code,
    const FString& Context,
    bool bAutoRecovered)
{
    // Create error info
    FSpacetimeDBErrorInfo ErrorInfo(Message, Severity, Category, Code, Context, bAutoRecovered);
    
    // Determine log level based on severity
    ELogVerbosity::Type LogVerbosity;
    FString SeverityString;
    
    switch (Severity)
    {
        case ESpacetimeDBErrorSeverity::Info:
            LogVerbosity = ELogVerbosity::Log;
            SeverityString = TEXT("INFO");
            break;
        case ESpacetimeDBErrorSeverity::Warning:
            LogVerbosity = ELogVerbosity::Warning;
            SeverityString = TEXT("WARNING");
            break;
        case ESpacetimeDBErrorSeverity::Error:
            LogVerbosity = ELogVerbosity::Error;
            SeverityString = TEXT("ERROR");
            break;
        case ESpacetimeDBErrorSeverity::Critical:
            LogVerbosity = ELogVerbosity::Error;
            SeverityString = TEXT("CRITICAL");
            break;
        case ESpacetimeDBErrorSeverity::Fatal:
            LogVerbosity = ELogVerbosity::Fatal;
            SeverityString = TEXT("FATAL");
            break;
        default:
            LogVerbosity = ELogVerbosity::Error;
            SeverityString = TEXT("UNKNOWN");
            break;
    }
    
    // Format log message
    FString LogMessage = FString::Printf(TEXT("[%s] %s"), *Category, *Message);
    
    // Add code if available
    if (Code != 0)
    {
        LogMessage += FString::Printf(TEXT(" (Code: %d)"), Code);
    }
    
    // Add context if available
    if (!Context.IsEmpty())
    {
        LogMessage += FString::Printf(TEXT(" - Context: %s"), *Context);
    }
    
    // Add auto-recovery information if applicable
    if (bAutoRecovered)
    {
        LogMessage += TEXT(" [Auto-recovered]");
    }
    
    // Log with appropriate verbosity
    UE_LOG(LogSpacetimeDB, Log, TEXT("%s: %s"), *SeverityString, *LogMessage);
    
    // For critical and fatal errors, also print to screen if possible
    if (Severity >= ESpacetimeDBErrorSeverity::Critical && GEngine)
    {
        // Determine color based on severity
        FColor MessageColor = (Severity == ESpacetimeDBErrorSeverity::Critical) ? FColor::Red : FColor::Purple;
        
        // Display on screen
        GEngine->AddOnScreenDebugMessage(-1, 10.0f, MessageColor, 
            FString::Printf(TEXT("SpacetimeDB %s: %s"), *SeverityString, *Message));
    }
    
    return ErrorInfo;
}

FSpacetimeDBErrorInfo FSpacetimeDBErrorHandler::HandleFFIError(
    const FString& FunctionName,
    const FString& ErrorMessage,
    bool bLogStackTrace)
{
    // Try to parse detailed error info from the message
    FSpacetimeDBErrorInfo ErrorInfo;
    if (!ParseFFIErrorMessage(ErrorMessage, ErrorInfo))
    {
        // If parsing failed, create a generic error
        ErrorInfo = FSpacetimeDBErrorInfo(
            ErrorMessage,
            ESpacetimeDBErrorSeverity::Error,
            TEXT("FFI"),
            0,
            FString::Printf(TEXT("Function: %s"), *FunctionName)
        );
    }
    
    // Always set the function name in the context
    if (ErrorInfo.Context.IsEmpty())
    {
        ErrorInfo.Context = FString::Printf(TEXT("Function: %s"), *FunctionName);
    }
    else if (!ErrorInfo.Context.Contains(FunctionName))
    {
        ErrorInfo.Context += FString::Printf(TEXT(", Function: %s"), *FunctionName);
    }
    
    // Get stack trace if requested
    FString StackTrace;
    if (bLogStackTrace)
    {
        StackTrace = FString::Printf(TEXT("\nStack trace:\n%s"), *FStackTrace::GetStackTrace(2)); // Skip this and calling function
    }
    
    // Log the error
    UE_LOG(LogSpacetimeDB, Error, TEXT("FFI Error in %s: %s%s"), 
        *FunctionName, *ErrorMessage, bLogStackTrace ? *StackTrace : TEXT(""));
    
    return ErrorInfo;
}

bool FSpacetimeDBErrorHandler::ParseFFIErrorMessage(const FString& ErrorMessage, FSpacetimeDBErrorInfo& OutInfo)
{
    // Default initialization
    OutInfo = FSpacetimeDBErrorInfo(
        ErrorMessage,
        ESpacetimeDBErrorSeverity::Error,
        TEXT("FFI")
    );
    
    // Check for empty message
    if (ErrorMessage.IsEmpty())
    {
        OutInfo.Message = TEXT("Unknown error (empty message)");
        return false;
    }
    
    // Try to parse structured error messages in format: [Category] Message (Code)
    // Or any subset of these components
    
    int32 CategoryStart = ErrorMessage.Find(TEXT("["));
    int32 CategoryEnd = ErrorMessage.Find(TEXT("]"));
    
    // If we have a category section
    if (CategoryStart != INDEX_NONE && CategoryEnd != INDEX_NONE && CategoryEnd > CategoryStart)
    {
        OutInfo.Category = ErrorMessage.Mid(CategoryStart + 1, CategoryEnd - CategoryStart - 1).TrimStartAndEnd();
        
        // Message starts after the category
        FString RemainingMessage = ErrorMessage.Mid(CategoryEnd + 1).TrimStartAndEnd();
        
        // Check for error code in parentheses at the end
        int32 CodeStart = RemainingMessage.Find(TEXT("("), ESearchCase::IgnoreCase, ESearchDir::FromEnd);
        int32 CodeEnd = RemainingMessage.Find(TEXT(")"), ESearchCase::IgnoreCase, ESearchDir::FromEnd);
        
        if (CodeStart != INDEX_NONE && CodeEnd != INDEX_NONE && CodeEnd > CodeStart)
        {
            FString CodeStr = RemainingMessage.Mid(CodeStart + 1, CodeEnd - CodeStart - 1).TrimStartAndEnd();
            if (CodeStr.IsNumeric())
            {
                OutInfo.Code = FCString::Atoi(*CodeStr);
                OutInfo.Message = RemainingMessage.Left(CodeStart).TrimStartAndEnd();
            }
            else
            {
                // If it's not a numeric code, treat the whole string as the message
                OutInfo.Message = RemainingMessage;
            }
        }
        else
        {
            // No code found, the remaining is the message
            OutInfo.Message = RemainingMessage;
        }
        
        return true;
    }
    
    // Check for severity indicators in the message
    if (ErrorMessage.Contains(TEXT("FATAL"), ESearchCase::IgnoreCase))
    {
        OutInfo.Severity = ESpacetimeDBErrorSeverity::Fatal;
    }
    else if (ErrorMessage.Contains(TEXT("CRITICAL"), ESearchCase::IgnoreCase))
    {
        OutInfo.Severity = ESpacetimeDBErrorSeverity::Critical;
    }
    else if (ErrorMessage.Contains(TEXT("ERROR"), ESearchCase::IgnoreCase))
    {
        OutInfo.Severity = ESpacetimeDBErrorSeverity::Error;
    }
    else if (ErrorMessage.Contains(TEXT("WARNING"), ESearchCase::IgnoreCase))
    {
        OutInfo.Severity = ESpacetimeDBErrorSeverity::Warning;
    }
    else if (ErrorMessage.Contains(TEXT("INFO"), ESearchCase::IgnoreCase))
    {
        OutInfo.Severity = ESpacetimeDBErrorSeverity::Info;
    }
    
    // If we couldn't parse the structure, just use the full message
    return false;
} 