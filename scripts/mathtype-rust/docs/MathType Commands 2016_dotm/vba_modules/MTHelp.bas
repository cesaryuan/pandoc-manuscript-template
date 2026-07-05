Attribute VB_Name = "MTHelp"
'====================================================================
' (c) Copyright 1992-2011 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTHelp.bas 14    5/06/14 9:54a Jimm $
'====================================================================

' this module is shared between Word and PowerPoint

' constants for HtmlHelp
Public Const HH_DISPLAY_TOPIC = &H0
Public Const HH_HELP_CONTEXT = &HF

'Constants for use in Help calls from dialogs
#If Win32 Then
Public Const hlpMSWDEquation_Number_Format_Dialog = 6300
Public Const hlpMSWDFormat_Equations_Dialog = 6500
Public Const hlpMSWDSet_Equation_Preferences_Dialog = 37
Public Const hlpMSWDConvert_Equations_Dialog = 44
Public Const hlpMSWDInsert_Equation_Section_Dialog = 114
Public Const hlpMSWDFormat_Equation_Section_Dialog = 116
Public Const hlpMSWDInsert_Equation_Number_Dialog = 118
Public Const hlpMSWDInsert_Requation_Ref_Dialog = 119
Public Const hlpMSWDExport_Equations_Dialog = 123
Public Const hlpMSWDExport_MathPage_Dialog = 124
Public Const hlpMSWDOptions_MTOptions_Dialog = 3174
Public Const hlpMSWDPaste_Pref_Dialog = 3175
Public Const hlpMSWDPreferences_Dialog = 3249
#Else
Public Const hlpMSWDEquation_Number_Format_Dialog = 3255
Public Const hlpMSWDFormat_Equations_Dialog = 3256
Public Const hlpMSWDSet_Equation_Preferences_Dialog = 3249
Public Const hlpMSWDConvert_Equations_Dialog = 3257
Public Const hlpMSWDInsert_Equation_Section_Dialog = 3252
Public Const hlpMSWDFormat_Equation_Section_Dialog = 3253
Public Const hlpMSWDInsert_Equation_Number_Dialog = 3254
Public Const hlpMSWDInsert_Requation_Ref_Dialog = 3251
Public Const hlpMSWDExport_Equations_Dialog = 3258
Public Const hlpMSWDExport_MathPage_Dialog = 3259
Public Const hlpMSWDOptions_MTOptions_Dialog = 3210
Public Const hlpMSWDPaste_Pref_Dialog = 3211
Public Const hlpMSWDPreferences_Dialog = 3249
#End If

' used externally by UI template macros
' uses the regsitry to pass arguments
Public Sub MTHelpTopicViaRegistry()
    #If Win32 Then
    MTHelp.MTHelpTopic Val(GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, "HelpTopicArgument"))
    #Else
    MTHelpLaunch Val(GetPreference(HKEY_CURRENT_USER, mtreg_MT_WORDCMDS_LOCATION, "HelpTopicArgument"))
    #End If
End Sub

'Opens the MathType Help file to a particular topic
'topic = help constant from HELP.H (0 for index)
Public Sub MTHelpTopic(topic As Long)
    Dim cmd As Long
    Dim result As Long
    Dim helpFileName As String

    If topic = 0 Then
        cmd = HH_DISPLAY_TOPIC 'HELP_INDEX
    Else
        cmd = HH_HELP_CONTEXT 'HELP_CONTEXT
    End If

    helpFileName = GetMTHelpFile()
#If Win32 Then
    If helpFileName <> "" Then
        'result = WinHelp(0, helpFileName, cmd, topic)
        result = HtmlHelp(0, helpFileName, cmd, topic)
    Else
        MsgBox MTLib.GetUserString2("1610", "3210", "The MathType help file couldn't be found"), _
            vbCritical, MTLib.GetUserString2("1611", "3211", "MathType Help Problem")
    End If
#End If
End Sub

'Gets the location of MathType's Help file from the registry
Public Function GetMTHelpFile() As String

    Dim path As String, helpFileName As String

    'get the location of Mathtype from the registry
    path = GetMTHelpFilePath()

    'Get the name of the current help file from MathType's registry
    helpFileName = GetMTHelpFileName()

    If helpFileName <> "" And path <> "" Then
        GetMTHelpFile = path & "\" & helpFileName
    Else
        GetMTHelpFile = ""
    End If

End Function

Private Function GetMTHelpFilePath() As String
    GetMTHelpFilePath = GetPreference(HKEY_LOCAL_MACHINE, mtreg_MT_HKLM_DIRECTORIES, mtreg_MT_HELPDIR_KEY)
End Function

Private Function GetMTHelpFileName() As String
    GetMTHelpFileName = GetPreference(HKEY_LOCAL_MACHINE, mtreg_MT_HELPFILE_LOCATION, mtreg_MT_HELPFILE_KEY)
End Function
