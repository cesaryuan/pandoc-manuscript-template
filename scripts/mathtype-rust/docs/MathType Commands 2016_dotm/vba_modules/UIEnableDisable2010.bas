Attribute VB_Name = "UIEnableDisable2010"
'====================================================================
' (c) Copyright 2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/OfficeVersion/2010/UIEnableDisableWord2010.bas 1     10/11/11 2:05p Jimm $
'====================================================================

Public Function IsNotUnsupportedView() As Boolean

#If Win32 Then
    IsNotUnsupportedView = (ActiveProtectedViewWindow Is Nothing)
#Else
    IsNotUnsupportedView = True
    If (ActiveWindow.View = wdNotesView Or ActiveWindow.View = wdPublishingView) Then
        IsNotUnsupportedView = False
    End If
#End If
        
End Function

Public Function NoDirectCall_GenEnabledByAppFunctionality(id As String) As Boolean
    
    NoDirectCall_GenEnabledByAppFunctionality = True
    
    AutoExec.PrivateMain ' Execute main to initialize MathType

    If buttonStates Is Nothing Then
        InitializeStatesCollection
    End If
    
#If Win32 Then
    If buttonStates(id) And NotInUnsupportedView Then
        If Not Application.ActiveProtectedViewWindow Is Nothing Then
            NoDirectCall_GenEnabledByAppFunctionality = False
            Exit Function
        End If
    End If
#End If
    
    If buttonStates(id) And IsFunctionalityOK Then
        NoDirectCall_GenEnabledByAppFunctionality = currentStatesCollection(CStr(IsFunctionalityOK))
    End If
    
End Function
