Attribute VB_Name = "CommandGuard"
Attribute VB_Base = "0{FCFB3D2A-A0FA-1068-A738-08002B3371B5}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False

'CommandGuard
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/CommandGuard.cls 14    9/17/12 1:21p Jimm $
'=====================================================================

' This file is shared between PP and Word

Private Sub Class_Initialize()

    On Error Resume Next

    ' Code to handle simultaneous execution
    If macroExecuting = True Then
        MsgBox "Only One MathType command can be executed at a time.  Please try again later."
        'Increment the OffBusy version check variable
        err.Raise DSI_ABORT_EXCEPTION
    Else
        macroExecuting = True
    End If

    ' initialize API
    Dim stat As Boolean
    Dim dllver As Long
    Dim msg As String
    Dim result As Long
    result = mpMTDLL_NOT_FOUND

    stat = False
    SetDLLPath

    #If Win32 Then
    Const ErrorVal As Integer = 0
    #Else
    Const ErrorVal As Integer = -1
    #End If

    ' Attempt to load the DLL
    result = MTInitAPI(mtinitLAUNCH_AS_NEEDED, 30)
        
    If result = mpMTDLL_NOT_FOUND Then
        ShowDLLNotFoundError
    ElseIf result <= ErrorVal Then
       msg = ULib.GetUserString2("1606", "3206", "The MathType commands could not communicate with MathType. There was a problem starting the API. Please be sure that MathType is properly installed.")
    Else
        'get the API Version (loads DLL)
        dllver = MTAPIVersion(MTAPI_VERSION)
        If dllver = mpBAD_VERSION Then
            ShowDLLNotFoundError
        'check the version against our constants
        ElseIf (dllver > mtversMajVerHi) Or (dllver < mtversMajVerLo) Then
            msg = ULib.GetUserString2("1607", "3207", "The version of this macro doesn't match the version of MathType's DLL. Reinstall MathType to fix this condition.")
        ElseIf (dllver < mtversMinVer) Then
            msg = ULib.GetUserString2("1608", "3208", "A more recent version of MathType's DLL is required to use this macro. Reinstall MathType to fix this condition.")
        Else
            stat = True
        End If
    End If

    If Not stat Then
        EndAPI
        If (Len(msg) > 0) Then
            MsgBox msg, vbCritical, ULib.GetUserString2("1609", "3209", "MathType Commands for Microsoft Word Error")
        End If
        err.Raise DSI_ABORT_EXCEPTION
    End If
End Sub

Private Sub Class_Terminate()
    EndAPI
End Sub

Private Sub EndAPI()

    On Error Resume Next

    ' code to release the execution lock
    macroExecuting = False

    ' terminate API
    MTTermAPI

End Sub
