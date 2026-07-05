Attribute VB_Name = "MTSetEqnPrefs"
'MTSetEqnPrefs: 5.0
'=====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/MTSetEqnPrefs.bas 30    10/11/11 2:12p Jimm $
'=====================================================================

'This macro retrieves a set of equation preferences and sets the current
'document to use these preferences for all new equations. The preference
'data is stored in the Custom Document Property "MTPreferences"

Option Explicit

'Global variables for communication with dialog
Public gDlgCanceled As Boolean  'True if user cancelled dialog
Public gUseMTPrefs As Boolean   'True to use MathType settings
Public gMTPrefFile$             'Name of preference file loaded (filename only)
Public gMTPrefs$                'Preferences in one string

Public Sub DlgMain()
Attribute DlgMain.VB_Description = "Set equation preferences for new equations."
    Dim title As String
    
    title = MTLib.GetUserString2("0800", "2400", "!0800Set Equation Preferences")
    'check to see if a document is open
    If MTLib.IsDocumentOpen(title) = False Then
        GoTo abort
    End If

    'check if this command is allowed in the current view
    If MTLib.IsCurrentViewOK(title) = False Then
        GoTo abort
    End If
    
    'Put up the dialog and get results
    MTSetEqPrefs.Show

    'If the dialog is not cancelled
    If Not gDlgCanceled Then
        If gUseMTPrefs = True Then
            MTLib.WriteDocPropString ActiveDocument, mtprop_USE_MATHTYPE_PREFS, "1"
        Else
            MTLib.DeleteDocProperty ActiveDocument, mtprop_USE_MATHTYPE_PREFS
            MTIncrementStatisticBy "CSetEqPref", 1
        End If
        
        'if the user created or changed the document's settings, save them
        'regardless of which option they chose
        If gMTPrefs$ <> "" And gMTPrefFile$ <> "" Then
            MTLib.WriteDocPropString ActiveDocument, mtprop_PREFERENCES, gMTPrefs$
            MTLib.WriteDocPropString ActiveDocument, mtprop_PREFERENCES_FILE, gMTPrefFile$
        End If
    End If

abort:
End Sub
