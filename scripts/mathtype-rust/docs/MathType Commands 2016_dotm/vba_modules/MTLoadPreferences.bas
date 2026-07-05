Attribute VB_Name = "MTLoadPreferences"
'MTLoadPreferences: 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTLoadPreferences.bas 25    1/19/10 2:29p Jimm $
'=====================================================================
'Upgrades MathType 3.x preference scheme
'The old scheme stored filenames in AutoText macros and would
'auto-launch macro at startup to load into MathType. Since
'the new scheme doesn't need to do anything on loading the document, this
'macro will grab the preference info from the file it was stored in, place
'it into the template, then remove the statements in the Auto macros that
'launch this macro.

Option Explicit

Public Sub DlgMain()
Attribute DlgMain.VB_Description = "Run only by older MathType macros, converts to newer preferences scheme."
    Dim zMathTypePrefFile As String, prefStr As String, file As String
    Dim context As Long
    Dim stat As Long
    
    'initialize the DLL and check to make sure it's the right version
    If Not MathPage.IsWLLVersionOK Then GoTo abort

    'determine the context of our autotext entry
    context = DetermineContext("ZMathTypePrefFile")

    'if the the string actually exists in any location, let's take care of it
    Select Case context
    Case 0      'Normal template
        'get the file name
        zMathTypePrefFile = NormalTemplate.AutoTextEntries("ZMathTypePrefFile").value
        'get the string
        prefStr = MTLib.GetPrefsFromFile(zMathTypePrefFile)
        'store the string
        If prefStr <> "" Then
            stat = MTLib.WriteDocPropString(NormalTemplate, _
                mtprop_PREFERENCES, prefStr)
            'set the preference source in template to Document
            file = MTLib.GetFileNameFromPath(zMathTypePrefFile)
            stat = MTLib.WriteDocPropString(NormalTemplate, _
                mtprop_PREFERENCES_FILE, file)
            MTLib.DeleteDocProperty NormalTemplate, mtprop_USE_MATHTYPE_PREFS
        End If
        'delete the old AutoText Entry
        NormalTemplate.AutoTextEntries("ZMathTypePrefFile").delete
        MTLib.EditAutoMacros NormalTemplate, "AutoNew"
        MTLib.EditAutoMacros NormalTemplate, "AutoOpen"
    Case 1      'attached template
        zMathTypePrefFile = ActiveDocument.AttachedTemplate.AutoTextEntries("ZMathTypePrefFile").value
        'get the string
        prefStr = MTLib.GetPrefsFromFile(zMathTypePrefFile$)
        If prefStr <> "" Then
            'store the string
            stat = MTLib.WriteDocPropString(ActiveDocument, _
                mtprop_PREFERENCES, prefStr)
            'set the preference source in template to Document
            file = MTLib.GetFileNameFromPath(zMathTypePrefFile)
            MTLib.DeleteDocProperty ActiveDocument, mtprop_USE_MATHTYPE_PREFS
        End If
        'Delete the old AutoText Entry
        ActiveDocument.AttachedTemplate.AutoTextEntries("ZMathTypePrefFile").delete
        MTLib.EditAutoMacros ActiveDocument.AttachedTemplate, "AutoNew"
        MTLib.EditAutoMacros ActiveDocument.AttachedTemplate, "AutoOpen"
    Case Else
        'There was an automacro somewhere, but no preferences. Just remove the
        'automatic run statements from both sources, since they are useless
        MTLib.EditAutoMacros NormalTemplate, "AutoNew"
        MTLib.EditAutoMacros NormalTemplate, "AutoOpen"
        MTLib.EditAutoMacros ActiveDocument.AttachedTemplate, "AutoNew"
        MTLib.EditAutoMacros ActiveDocument.AttachedTemplate, "AutoOpen"
    End Select
    Application.StatusBar = ""

abort:
    MTTermAPI
End Sub

'Returns a 0 if the AutoText entry is in the Normal Template and
'a 1 if the AutText entry is in the attached template.
'If no autotext entry in either location -1 is returned.
Private Function DetermineContext(GlossaryItem As String) As Long
    Dim itExists As Boolean
    Dim oneEntry As AutoTextEntry

    itExists = False
    'Check to see if it is in the attached template
    For Each oneEntry In ActiveDocument.AttachedTemplate.AutoTextEntries
        If oneEntry.name = GlossaryItem Then
            itExists = True
        End If
    Next oneEntry
    
    If itExists Then
        DetermineContext = 1
    Else
        'since it wasn't in the attached template, try NORMAL.DOT
        For Each oneEntry In NormalTemplate.AutoTextEntries
            If oneEntry.name = GlossaryItem Then
                itExists = True
            End If
        Next oneEntry
        If itExists Then
            DetermineContext = 0
        Else
            'It wasn't in either one, so return an empty string.
            DetermineContext = -1
        End If
    End If
End Function

