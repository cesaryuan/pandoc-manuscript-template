Attribute VB_Name = "UILib"
'UILib 5.2
'====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/UILib.bas 107   5/06/14 9:54a Jimm $
'====================================================================

' Note: * all functions/subs that start with MTCommand_ are "top-level" entry points called from
'         OnAction handlers (e.g. buttons, menu items, etc).
'       * all functions/subs that start with MTCallback_ are are called by
'         Word 2007 as needed to refresh the Ribbon UI
'       * all functions/subs that start with NoDirectCall_
'          MUST NOT BE CALLED except via a RunXXX command

' variable for dispatch class
Private cls

' module name
Private Const module As String = "UILib"
Private Const moduleCLS As String = "UILibCls"

' browse variables
Public browseChoice As Long
Public forward As Boolean

Public gMTEqnNumsOnRight As Boolean

Public Sub MTCommand_MTBrowseNext(Optional choice)
    If IsMissing(choice) Then
        browseChoice = -1
    End If
    forward = True
    RunMTDLLCommand moduleCLS, "NoDirectCall_Browse"
End Sub

Public Sub MTCommand_MTBrowsePrevious(Optional choice)
    If IsMissing(choice) Then
        browseChoice = -1
    End If
    forward = False
    RunMTDLLCommand moduleCLS, "NoDirectCall_Browse"
End Sub

'Runs Set Equation Reference command
Public Sub MTPlaceRef()
    RunMTDLLCommand "", kMTCommands & ".MTPlaceRef.DlgMain", False
End Sub

'Runs Modify Chapter/Section Break command
Public Sub MTEditEquationSection()
    RunMTDLLCommand "", kMTCommands & ".MTFormatEquationSection.EditEquationSection", False
End Sub

'Runs Modify Chapter/Section Break command
Public Sub MTEditEquationSection2()
    RunMTDLLCommand "", kMTCommands & ".MTFormatEquationSection.EditEquationSection", False
End Sub

'Runs Modify Chapter/Section Break command
Public Sub MTEditEquationSection3()
    RunMTDLLCommand "", kMTCommands & ".MTFormatEquationSection.NewVersion", False
End Sub

'Runs Modify Chapter/Section Break command
Public Sub MTEditEquationSection4()
    RunMTDLLCommand "", kMTCommands & ".MTFormatEquationSection.NewVersion", False
End Sub

'Runs Modify Chapter/Section Break command
Public Sub MTEditEquationSection5()
    RunMTDLLCommand "", kMTCommands & ".MTFormatEquationSection.NewVersion", False
End Sub

'Runs Export All MathPage command
Public Sub ExportAllMathPage()
    RunMTDLLCommand "", kMTCommands & ".MathPage.ExportAll", False
End Sub

'Runs Export to MathPage command
Public Sub MTCommand_ExportMathPage()
    RunMTDLLCommand "", kMTCommands & ".MathPage.MP_ExportTo", False, mtbIDMathPage
End Sub

'Runs Set Equation Preferences command
Public Sub MTCommand_SetEqnPrefs()
    RunMTDLLCommand "", kMTCommands & ".MTSetEqnPrefs.DlgMain", False, mtbIDSetEqnPrefs
End Sub

'Runs Insert Inline Equation command
Public Sub MTCommand_InsertInlineEqn()
    RunMTDLLCommand moduleCLS, "NoDirectCall_InsertInlineEqn", True, mtbIDInsInlineMTEqn
End Sub

'Runs Insert Display Equation command
Public Sub MTCommand_InsertDispEqn()
    RunMTDLLCommand moduleCLS, "NoDirectCall_InsertDispEqn", True, mtbIDInsDispMTEqn
End Sub

'Runs Insert Left-Numbered Display Equation command
Public Sub MTCommand_InsertLeftNumberedDispEqn()
    RunMTDLLCommand moduleCLS, "NoDirectCall_InsertLeftNumberedDispEqn", True, mtbIDInsDispMTEqnLeftNum
End Sub

'Runs Insert Right-Numbered Display Equation command
Public Sub MTCommand_InsertRightNumberedDispEqn()
    RunMTDLLCommand moduleCLS, "NoDirectCall_InsertRightNumberedDispEqn", True, mtbIDInsDispMTEqnRightNum
End Sub

'Runs Insert Left-Numbered Display Equation command
Public Sub MTCommand_InsertLeftNumberedDispEqnEB()
    RunMTDLLCommand moduleCLS, "NoDirectCall_InsertLeftNumberedDispEqnEB"
End Sub

'Runs Insert Right-Numbered Display Equation command
Public Sub MTCommand_InsertRightNumberedDispEqnEB()
    RunMTDLLCommand moduleCLS, "NoDirectCall_InsertRightNumberedDispEqnEB"
End Sub

'Runs Insert Equation Number command
Public Sub MTCommand_InsertEqnNum()
    RunMTDLLCommand moduleCLS, "NoDirectCall_InsertEqnNum", True, mtbIDInsertNumber
End Sub

'Runs Insert Equation Reference command
Public Sub MTCommand_InsertEqnRef()
    RunMTDLLCommand moduleCLS, "NoDirectCall_InsertEqnRef", True, mtbIDEquationReference
End Sub

'Runs Insert Equation Section command
Public Sub MTCommand_InsertSecNum()
    RunMTDLLCommand "", kMTCommands & ".MTSecNum.DlgMain", False, mtbIDMoreBreaks
End Sub

'Runs Insert Equation Section command
Public Sub MTCommand_InsertNextSection()
    RunMTDLLCommand "", kMTCommands & ".MTSecNum.InsertNextSectionBreak", False, mtbIDInsertNextSection
End Sub

'Runs Insert Equation Section command
Public Sub MTCommand_InsertNextChapter()
    RunMTDLLCommand "", kMTCommands & ".MTSecNum.InsertNextChapterBreak", False, mtbIDInsertNextChapter
End Sub

'Runs Modify Equation Section command
Public Sub MTCommand_FormatEqnSec()
    RunMTDLLCommand "", kMTCommands & ".MTFormatEquationSection.DlgMain", False, mtbIDModifyBreak
End Sub

'Runs Format Equation Numbers command
Public Sub MTCommand_FormatEqnNum()
    RunMTDLLCommand "", kMTCommands & ".MTEqnNumFormat.DlgMain", False, mtbIDFormatEqnNums
End Sub

'Runs Format Equations command
Public Sub MTCommand_FormatEqns()
    RunMTDLLCommand "", kMTCommands & ".MTFormatEquations.DlgMain", False, mtbIDFormatEqns
End Sub

'Runs Convert Equations command
Public Sub MTCommand_ConvertEqns()
    RunMTDLLCommand "", kMTCommands & ".MTConvertEquations.DlgMain", False, mtbIDConvertEqns
End Sub

'Runs Export Equations command
Public Sub MTCommand_ExportEqns()
    RunMTDLLCommand "", kMTCommands & ".MTExportEquations.DlgMain", False, mtbIDExportEqns
End Sub

'Runs Update Fields command
Public Sub MTCommand_UpdateEqns()
    RunMTDLLCommand moduleCLS, "NoDirectCall_UpdateEqns", True, mtbIDUpdateEqnNums
End Sub

Public Sub MTCommand_TeXToggle()
   RunMTDLLCommand "", kMTCommands & ".MTTeXToggle.DlgMain", False, mtbIDTeXToggle
End Sub

Public Sub MTCommand_EditEquationOpen()
    RunMTDLLCommand "", kMTCommands & ".MTLib.MTEditEquationOpen", False
End Sub

Public Sub MTCommand_EditEquationInPlace()
    RunMTDLLCommand "", kMTCommands & ".MTLib.MTEditEquationInPlace", False
End Sub

Public Sub MTCommand_MathInputControl()
    RunMTDLLCommand "", kMTCommands & ".MTMathInputControl.DlgMain", False
End Sub

Public Sub MTCommand_MTOptions()
    RunMTDLLCommand "", kMTCommands & ".MTOptions.DlgMain", False, mtbIDMTOptions
End Sub

'Runs a synchronous command (requiring the DLL) inside an error handler
Public Sub RunMTDLLCommand(module As String, command As String, Optional isLocal As Boolean = True, Optional btnID As String = "")
    RunMTDLLCode module, command, "command", isLocal, btnID
End Sub

'Runs a asynchronous command (requiring the DLL) inside an error handler
Public Sub RunDocCallback(module As String, command As String, Optional isLocal As Boolean = True, Optional btnID As String = "")
    RunMTDLLCode module, command, "callback", isLocal, btnID
End Sub

'Runs code (requiring the DLL) inside an error handler with a guard
Private Sub RunMTDLLCode(module As String, command As String, calltype As String, Optional isLocal As Boolean = True, Optional btnID As String = "")
    On Error GoTo abort

    ' Instantiate the guard object
    Dim toplevelguard As Object
    If calltype = "callback" Then
        Set toplevelguard = New CallbackGuard
    Else
        Set toplevelguard = New CommandGuard
    End If

    If LoadMathTypeCommands() Then
        RunDispatch module, command, isLocal, btnID
    End If

    Exit Sub

abort:
    WriteLog "RunMTDLLCode error:" & module & ":" & command & ":" & calltype
    ' generic automation error
    If err.Number = 440 Then
        Dim errNum
        Dim errDesc

        If (Not cls Is Nothing) And (Not IsEmpty(cls)) Then
            errNum = cls.ErrorNumber
            errDesc = cls.ErrorDescription
        End If
        ' extract the real error from the class
        Asserts.LogAndAlert module, "RunMTDLLCode", errNum & " " & errDesc
    ElseIf err.Number <> DSI_ABORT_EXCEPTION Then
        Dim desc As String
        desc = err.Description
        ' We use AssertFailure to notify the end user of the unexpected exception
        ' and to log the error.  However, since AssertFailure raises another
        ' DSI_ABORT_EXCEPTION, we need to set things up to resume after the assert and exit
        Asserts.LogAndAlert module, "Error running cmd=" & command & " mod=" & module & " in RunMTDLLCommand", desc
    End If
    ' No action needed for DSI_ABORT_EXCEPTION errors
    ' Exiting the subroutine will clear the error
End Sub

' dispatches a command to its associated class
Public Sub RunDispatch(module As String, command As String, Optional isLocal As Boolean = True, Optional btnID As String = "")
    ' We used to enable/disable the UI controls based on document state.
    ' Now we just bail out and display an error message if we cannot
    ' run our commands due to an invalid document state
    ' note that an empty string for btnID skips checking
    If Len(btnID) <> 0 Then
        UpdateState
        If Not NoDirectCall_IsEnabled(btnID) Then
            Dim msg As String
            msg = GetDisabledString(btnID)
            If Len(msg) = 0 Then
                MsgBox "Unable to execute this command"
            Else
                MsgBox GetUserString("!3014This command cannot be executed because ") & msg
            End If
            Exit Sub
        End If
    End If

    If isLocal Then
        Select Case module
            Case "AutoExecCls"
                Set cls = New AutoExecCls
            Case "UIHelpCls"
                Set cls = New UIHelpCls
            Case "UILibCls"
                Set cls = New UILibCls
            Case "UIWrappersCls"
                Set cls = New UIWrappersCls
            Case "UIRibbonCls"
                Set cls = New UIRibbonCls
        End Select
        If (Not cls Is Nothing) And (Not IsEmpty(cls)) Then
            On Error GoTo CallError
               CallByName cls, command, VbMethod
            GoTo done
CallError:
            WriteLog "RunDispatch error (1):" & command
        End If
    Else
        If module <> "" Then
            Asserts.AssertFailure "UILib", "RunDispatch", "Can't dispatch to unkown module" & module
            WriteLog "RunDispatch error (2):" & command
        Else
            ' on the Mac, Application.Run is a Sub, not a Function
            #If Win32 Then
            command = replace(command, ".", "_", , , vbTextCompare)
            'Set cls = Application.Run("new_MTCommandsDispatchClass")
            Set cls = New MTCommandsDispatchCls
            WriteLog "sanity checking cls"
            If (Not cls Is Nothing) And (Not IsEmpty(cls)) Then
                WriteLog "Calling " & command
                    CallByName cls, command, VbMethod ' this method is unavailable on the Mac
            End If
            #Else
                On Error GoTo AppRunError
                Application.Run command
AppRunError:
            #End If
        End If
    End If
done:
End Sub

'Runs a function (requiring the DLL) inside an error handler,
'but does not execute LoadMathTypeCommands
Public Function RunUICallback(command As String, ParamArray args() As Variant)
    On Error GoTo abort

    ' Instantiate the guard object
    Dim toplevelguard As CallbackGuard
    Set toplevelguard = New CallbackGuard

    Select Case command
        Case "NoDirectCall_GetUnlockScreenTipUI"
            RunUICallback = NoDirectCall_GetUnlockScreenTipUI()
        Case "NoDirectCall_GetUnlockUI"
            RunUICallback = NoDirectCall_GetUnlockUI()
        Case "NoDirectCall_IsEnabled"
            RunUICallback = NoDirectCall_IsEnabled((args(0)))
        Case "NoDirectCall_LocateSupertip"
            RunUICallback = NoDirectCall_LocateSupertip((args(0)))
        Case "NoDirectCall_GetNumEqVisible"
            RunUICallback = NoDirectCall_GetNumEqVisible((args(0)))
        Case "NoDirectCall_GenEnabledByAppFunctionality"
            RunUICallback = NoDirectCall_GenEnabledByAppFunctionality((args(0)))
        Case Else
            WriteLog "RunUICallback error 1:" & command
    End Select

    Exit Function
abort:
    WriteLog "RunUICallback error 2:" & command
    If err.Number <> DSI_ABORT_EXCEPTION Then
        Dim desc As String
        desc = err.Description
        ' We use AssertFailure to notify the end user of the unexpected exception
        ' and to log the error.  However, since AssertFailure raises another
        ' DSI_ABORT_EXCEPTION, we need to set things up to resume after the assert and exit
        On Error Resume Next
        Asserts.LogAndAlert module, "RunUICallback", desc
    End If
    ' No action needed for DSI_ABORT_EXCEPTION errors
    ' Exiting the subroutine will clear the error
End Function

'Loads MathType Commands template if not already loaded
Function LoadMathTypeCommands() As Boolean
    LoadMathTypeCommands = True
End Function

Private Function HandleCommandUnavailable(addin As String) As Boolean
    Dim curPane As Pane
    On Error GoTo err
    If Val(Application.version) >= kWordX Then
        Set curPane = ActiveDocument.ActiveWindow.ActivePane
        ActiveDocument.Bookmarks("\StartOfDoc").Select
        AddIns.Add fileName:=addin, Install:=True
        curPane.Activate
        HandleCommandUnavailable = True
        Exit Function
    End If
err:
    WriteLog "HandleCommandUnavailable error"
    HandleCommandUnavailable = False
End Function

'Returns True if addIn/Template is already installed
Private Function IsAddInInstalled(addin As String) As Boolean

    IsAddInInstalled = False
    On Error GoTo err
    IsAddInInstalled = AddIns(addin).Installed
    If IsAddInInstalled = True Then Exit Function
err:
    On Error GoTo 0

    'one last attempt to brute force install the addin
    'as a fix for http://valor:8080/browse/MT-2085
    Dim currentAddin As addin
    Dim found As Boolean
    found = False

    For Each currentAddin In AddIns
        Dim currentAddinFullPath As String
        ' Sometimes AddIns don't respond well to .path or .name
        On Error GoTo skip
        currentAddinFullPath = currentAddin.path & Application.PathSeparator & currentAddin.name
        If (Strings.LCase(currentAddinFullPath) = Strings.LCase(addin)) And _
           (PathExists(currentAddinFullPath)) Then
            found = True
        End If
skip:
        On Error GoTo 0
    Next currentAddin

    If found = True Then
        AddIns(addin).Installed = True
        IsAddInInstalled = AddIns(addin).Installed
    End If

End Function

Private Function PathExists(path As String) As Boolean

    PathExists = False

    If Len(dir$(path)) > 0 And Len(path) > 0 Then
        PathExists = True
    End If

End Function

'Returns True if MT3 macros were found & removed
Public Function RemoveMT3Macros(aTemplate As Template) As Boolean
    RemoveMT3Macros = False

    'make sure we don't call it for our current templates
    'remove if MTLib exists, but "MTW4" AND "MT5API" do not exist
    If HasComponent(aTemplate, "MTLib") Then
        If Not HasComponent(aTemplate, "MTW4") Then
            If Not HasComponent(aTemplate, "MT5API") Then ' Attibute VB_Name for Mtapi.bas
                If Not HasComponent(aTemplate, "Declarations") Then
                    If LoadMathTypeCommands() Then
                        'Can't pass template, but we got it as activedoc.template
                        'and that's what MTCleanup does
                        WriteLog "Calling MTCleanup"
                        RunDispatch "", kMTCommands & ".MTLib.MTCleanup", False
                        RemoveMT3Macros = True
                    End If
                End If
            End If
        End If
    End If
End Function

'Returns True if component exists in the template
Private Function HasComponent(aTemplate As Template, name As String) As Boolean
    Dim temp As String

    On Error GoTo abort
    HasComponent = False
#If Win32 Then '.VBProject is unavailable on the Mac
    temp = aTemplate.VBProject.VBComponents(name).name
    HasComponent = True
#End If
abort:
End Function

'Takes a localizable string of the form "!nnnnString", where nnnn is a 4-digit string ID.
'If current language is English, or the language DLL can't be found, just strip the prefix
Public Function GetUserString(englishString As String) As String

    Dim buffer(1023) As Byte
    Dim tmpStr As String
    Dim bufLen As Long
   
    bufLen = 512
    MTGetUserWString englishString, buffer(0), bufLen
    tmpStr = buffer
    GetUserString = LeftB(tmpStr, bufLen * 2)

End Function

Public Function GetUserString2(winID As String, macID As String, englishString) As String

    Dim nnnnString As String
    Dim strID As String

    #If Win32 Then
        strID = winID
    #Else
        strID = macID
    #End If

    nnnnString = "!" + strID + englishString

    GetUserString2 = GetUserString(nnnnString)

End Function

'Displays error telling user to reinstall MathType
Public Sub ShowReinstallError()
    Beep
    MsgBox GetUserString("!0202An error occurred starting MathType's Commands for Word. Please re-install MathType."), _
        vbOKOnly, GetUserString("!0200MathType Commands")
End Sub

'Returns True if MathType's Full Functionality is available (i.e. not an expired demo)
Public Function IsFullFunctionality() As Boolean
    IsFullFunctionality = (MTIsFullFunctionality() = 1)
End Function

Public Sub CrashTest()
    RunMTDLLCommand moduleCLS, "NoDirectCall_CrashTest"
End Sub

'Callback routine that determines which split button for inserting numbered eqns is visible
Public Function NoDirectCall_GetNumEqVisible(id As String)
    NoDirectCall_GetNumEqVisible = False
    If id = "MathType_SB_NumEqnL" And gMTEqnNumsOnRight = False Then
        'Make left-default sb visible when the last numbered eqn was on left
        NoDirectCall_GetNumEqVisible = True
    ElseIf id = "MathType_SB_NumEqnR" And gMTEqnNumsOnRight = True Then
        'Make right-default sb visible when the last numbered eqn was on right
        NoDirectCall_GetNumEqVisible = True
    End If
End Function

' inits eqn numbering side from doc props, registry or default
Public Sub InitDefaultEqnNumSide()
    Dim regValue As String

    On Error GoTo missing
    ' default
    gMTEqnNumsOnRight = True

    ' if we find a per document setting, use it
    Dim Doc As Document
    Set Doc = ActiveDocument

    If Doc.CustomDocumentProperties.item(mtprop_EQN_NUMS_ON_RIGHT).value = True Then
        gMTEqnNumsOnRight = True
    ElseIf Doc.CustomDocumentProperties.item(mtprop_EQN_NUMS_ON_RIGHT).value = False Then
        gMTEqnNumsOnRight = False
    End If
    Exit Sub

missing:
    regValue = GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_EQN_NUMS_ON_RIGHT_KEY)
    Select Case regValue
    Case "1"
        gMTEqnNumsOnRight = True
    Case "0"
        gMTEqnNumsOnRight = False
    Case Else
        gMTEqnNumsOnRight = True
    End Select
End Sub

' saves the default side for equation numbering
Public Sub SaveDefaultEqnNumSide()

    ' save to custom doc props if we can, ignoring errors.
    On Error Resume Next
    Dim Doc As Document
    Set Doc = ActiveDocument

    If Not Doc.ReadOnly Then
        Doc.CustomDocumentProperties(mtprop_EQN_NUMS_ON_RIGHT).delete
        Doc.CustomDocumentProperties.Add mtprop_EQN_NUMS_ON_RIGHT, _
           False, msoPropertyTypeBoolean, gMTEqnNumsOnRight
    End If

    ' save to registry, but let error propagate to top-level handler if we encounter
    ' one since this should not happen
    On Error GoTo 0

    Dim textValue As String
    If gMTEqnNumsOnRight Then
        textValue = "1"
    Else
        textValue = "0"
    End If
    SetPreference HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, mtreg_MT_EQN_NUMS_ON_RIGHT_KEY, textValue
End Sub

'Returns True if the MT DLL version is OK, else displays error & returns False
Public Function IsDLLVersionOK()
    Dim stat As Boolean
    Dim result As Long
    Dim dllver As Long
    Dim msg As String

    'todo there should not be different return values on Mac vs Win
    #If Win32 Then
        Const ErrorVal As Integer = 0
    #Else
        Const ErrorVal As Integer = -1
    #End If

    stat = False
    result = MTInitAPI(mtinitLAUNCH_AS_NEEDED, 30)
    If result <= ErrorVal Then
        msg = UILib.GetUserString2("1606", "3206", "The MathType commands could not communicate with MathType. There was a problem starting the API. Please be sure that MathType is properly installed.")
    Else
        'get the API Version (loads DLL)
        dllver = MTAPIVersion(MTAPI_VERSION)
        If dllver = mpMTDLL_NOT_FOUND Or dllver = mpBAD_VERSION Then
            ShowDLLNotFoundError
        'check the version against our constants
        ElseIf (dllver > mtversMajVerHi) Or (dllver < mtversMajVerLo) Then
            msg = UILib.GetUserString2("1607", "3207", "The version of this macro doesn't match the version of MathType's DLL. Reinstall MathType to fix this condition.")
        ElseIf (dllver < mtversMinVer) Then
            msg = UILib.GetUserString2("1608", "3208", "A more recent version of MathType's DLL is required to use this macro. Reinstall MathType to fix this condition.")
        Else
            stat = True
        End If
    End If

    If Not stat And Len(msg) > 0 Then
        MsgBox msg, vbCritical, UILib.GetUserString2("1609", "3209", "MathType Commands for Microsoft Word Error")
    End If
    IsDLLVersionOK = stat
End Function

Public Function IsMathInputPanelAvailable() As Boolean

    IsMathInputPanelAvailable = False

#If Win32 Then
    ' there is no math input panel on the Mac
    Dim toplevelguard As Object
    Set toplevelguard = New CallbackGuard

    Dim isAvailable As Boolean
    MTIsMathInputPanelAvailable isAvailable
    If isAvailable Then
        IsMathInputPanelAvailable = True
    End If
#End If

End Function

Public Function GetMathTypeDir() As String

#If Win32 Then
    GetMathTypeDir = GetPreference(HKEY_LOCAL_MACHINE, mtreg_MT_HKLM_DIRECTORIES, mtreg_MT_PROGDIR_KEY)
#Else
    Dim path(1023) As Byte
    Dim pathLen As Long
    Dim stat As Long
    Dim tmpStr As String
    
    pathLen = 512

    stat = MTGetWPathToMathType(path(0), pathLen)
    If stat = mtOK Then
        tmpStr = path
        GetMathTypeDir = LeftB(tmpStr, pathLen * 2)
    Else
        GetMathTypeDir = ""
    End If
#End If
End Function
