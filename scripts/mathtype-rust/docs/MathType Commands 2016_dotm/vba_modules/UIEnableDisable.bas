Attribute VB_Name = "UIEnableDisable"
'====================================================================
' (c) Copyright 1992-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/UIEnableDisableWord.bas 55    7/27/14 2:33p Johns $
'====================================================================
' This module is used to enable and disable the user interface elements
' for Word depending on the state of the selection.

' The following functions and subroutines must exist in the PowerPoint
' and Word versions of this module.
'
' NoDirectCall_IsEnabled
' UpdateState
' NoDirectCall_LocateSupertip
' Invalidate
' UpdatePre2007CommandBar

' a collection of long values which are used with a bit mask to
' store the enabled state for a particular button/menu item. It uses
' a string key of button id's (one of the mtbIDXXX constants)
Public buttonStates As Collection
' a collection of possible states (i.e. the States enumeration values)
Private DocumentStates As Collection
' a collection of boolean values whose key is a string version of each States enumeration
' This is used to store the values from the current UI update cycle
Public currentStatesCollection As Collection
' a collection of boolean values whose key is a string version of each States enumeration
' this is used to store the value from the previous UI update cycle
Private previousStatesCollection As Collection
' a collection of strings whose key is is a string version of each States enumeration
' each string represents what should be shown in the disabled state
Private superTipDisabledCollection As Collection
' a collection of strings whose key is a button ID
' each string represents what should be shown in the enabled state
Private superTipEnabledCollection As Collection


' tests to see if the application is in a state where it's ok to
' execute a particular command. If the button should be enabled,
' it returns true, otherwise false.
'
' buttonID must be one of the mtbIDXXX constants defined in UIDeclarations
Public Function NoDirectCall_IsEnabled(buttonID As String) As Boolean
    NoDirectCall_IsEnabled = True ' must be initialized to true

    On Error GoTo err

    ' make sure we are ready to run
    If InitializeStatesCollection = False Then
        ' if initialization fails, then we need to bail
        GoTo err
    End If

    ' iterate through each document state
    Dim aState
    For Each aState In DocumentStates
        ' use the aState bitmask against the value stored in buttonStates
        ' to determine if the state really applies
        If buttonStates(buttonID) And aState Then
            ' during each loop check the current state value and apply it to
            ' the result. Once the loop completes the NoDirectCall_IsEnabled value
            ' should reflect if the button/menu item should be enabled
            NoDirectCall_IsEnabled = NoDirectCall_IsEnabled And currentStatesCollection(CStr(aState))
        End If
    Next
    Exit Function
err:
    ' failure results in a disabled button
    NoDirectCall_IsEnabled = False
End Function

' this function is called to update the currentStatesCollection with the
' current state. It does this by setting a boolean value for a given state.
'
' For example, the States.IsDocumentOpen enumeration value is used as a key,
' and the value associated with this key is the result of Not (Documents.Count = 0)
'
' the function returns false if the current state is the same as the previous state
' and true if the state has really changed
Public Function UpdateState() As Boolean

    WriteLog "UpdateState start"

    UpdateState = False

    On Error GoTo err

    ' make sure all fof the necessary collections are built
    If InitializeStatesCollection = False Then
        ' if initialization fails, we need to bail out
        GoTo err
    End If

    ' Most of the commands used here require an active document
    Dim hasDocuments As Boolean
    ' we reuse this value over and over as the result of checking some state
    Dim temp As Boolean

    hasDocuments = Not (Documents.count = 0)

    ' always enable state must always be set to true
    temp = True
    UpdateCollection currentStatesCollection, States.AlwaysEnable, temp

    'temp =
    'UpdateCollection currentStatesCollection, States.CheckSectionNumber, temp

    ' is document open state has the same value as the hasDocuments so just reuse it
    temp = hasDocuments
    UpdateCollection currentStatesCollection, States.IsDocumentOpen, temp

    If hasDocuments Then
        temp = ActiveDocument.saved And Len(ActiveDocument.path) > 0 And Not (ActiveDocument.ReadOnly)
    Else
        temp = False
    End If
    UpdateCollection currentStatesCollection, States.MathPageOK, temp

    ' make sure a document is open first before attempting to use a command
    ' many of the following states do the same thing
    If hasDocuments Then
        temp = ActiveDocument.Bookmarks.Exists("MTReference")
    Else
        temp = False
    End If
    UpdateCollection currentStatesCollection, States.MTReferenceExists, temp

    If hasDocuments Then
        temp = Not (Application.ActiveWindow.Selection.storyType = wdTextFrameStory)
    Else
        temp = False
    End If
    UpdateCollection currentStatesCollection, States.SelectionInTextBox, temp

    If hasDocuments Then
        temp = IsNotInReadingView()
    End If
    UpdateCollection currentStatesCollection, States.NotInReadingView, temp

    ' Warning: there appears to be a bug in pre-2007 versions of Word whereby
    ' accessing the read-only Selection.Information(wdXXX) method causes a
    ' DocumentChange event to fire.   Consequently, it is important that the
    ' App_DocumentChange event handler be fast, and that it does not indirectly
    ' invoke another call to UpdateState, or an infinite loop ensues.  See MT-1159
    If hasDocuments Then
        temp = Not (Application.ActiveWindow.Selection.Information(wdInCommentPane))
    Else
        temp = False
    End If
    UpdateCollection currentStatesCollection, States.Word97SelectionInCommentPane, temp

    If hasDocuments Then
        temp = Not (Application.ActiveWindow.Selection.Information(wdInFootnoteEndnotePane))
    Else
        temp = False
    End If
    UpdateCollection currentStatesCollection, States.Word97SelectionInFootnoteEndnotePane, temp

    If hasDocuments Then
        temp = Not (Application.ActiveWindow.Selection.Information(wdInHeaderFooter))
    Else
        temp = False
    End If
    UpdateCollection currentStatesCollection, States.Word97SelectionInHeaderFooter, temp

    If hasDocuments Then
        If Val(Application.version) = kWordX Then
            temp = IsWordXPActiveWindowViewSplitSpecial()
        Else
            temp = False
        End If
    Else
        temp = False
    End If
    UpdateCollection currentStatesCollection, States.WordXPActiveWindowViewSplitSpecial, temp


    temp = IsFullFunctionality
    UpdateCollection currentStatesCollection, States.IsFunctionalityOK, temp

    temp = IsNotUnsupportedView
    UpdateCollection currentStatesCollection, States.NotInUnsupportedView, temp

    ' now check the previous state with the current state to see if we should allow
    ' an update of the UI
    Dim aState
    For Each aState In DocumentStates
        If previousStatesCollection(CStr(aState)) <> currentStatesCollection(CStr(aState)) Then
            UpdateState = True
            Exit For
        End If
    Next

    ' update the previous state with the current state
    Dim aStateLong As Long
    For Each aState In DocumentStates
        aStateLong = aState
        UpdateCollection previousStatesCollection, aStateLong, currentStatesCollection(CStr(aStateLong))
    Next

    WriteLog "UpdateState end"

    Exit Function
err:
    ' if error occurs, then we can't be sure if the previous and new states are different
    ' so just return false so no further action is taken
    WriteLog "UpdateState error"
    UpdateState = False
End Function

' This function determines the super tip for a button
' It is nearly the same as NoDirectCall_IsEnabled, but it creates a string
' that partially describes the reason for the disabled state,
' or an empty string if the button is enabled
Public Function NoDirectCall_LocateSupertip(id As String) As String
    Dim disabledString As String

    On Error GoTo err

    disabledString = GetDisabledString(id)

    NoDirectCall_LocateSupertip = superTipEnabledCollection(id)

    ' For MTW6, we only set the supertip based on app functionality, since we aren't actually
    ' disabling buttons based on document state
    Dim isEnabled
    isEnabled = NoDirectCall_GenEnabledByAppFunctionality(id)
    If Not isEnabled Then
        If Len(NoDirectCall_LocateSupertip) <> 0 Then
            NoDirectCall_LocateSupertip = NoDirectCall_LocateSupertip & vbNewLine & vbNewLine
        End If
        NoDirectCall_LocateSupertip = NoDirectCall_LocateSupertip & UILib.GetUserString("!3000This button is disabled because: ") & disabledString & "."
    End If
    Exit Function
err:
    NoDirectCall_LocateSupertip = ""
End Function

Public Function GetDisabledString(id As String)
    Dim aCurrentState As Boolean

    On Error GoTo err

    ' make sure we are ready to run
    If InitializeStatesCollection = False Then
        ' if initialization fails, then we need to bail
        GoTo err
    End If

    ' We handle IsFunctionalityOK and IsDocumentOpen as special cases
    ' since when they hold, they should be the only reason given for being disabled
    If buttonStates(id) And States.IsFunctionalityOK Then
        If currentStatesCollection(CStr(States.IsFunctionalityOK)) = False Then
            GetDisabledString = superTipDisabledCollection(CStr(States.IsFunctionalityOK))
            Exit Function
        End If
    End If
    ' We also handle protected view as a special case. Note we check IsNotUnsupportedView()
    ' directly, since currentStatesCollection is not dependably updated in protected view
    ' We would need to add event listeners for protected view activation/open events and
    ' rewrite much of UpdateStates for protected view windows.
    If buttonStates(id) And States.NotInUnsupportedView Then
        If IsNotUnsupportedView() = False Then
            GetDisabledString = superTipDisabledCollection(CStr(States.NotInUnsupportedView))
            Exit Function
        End If
    End If
    If buttonStates(id) And States.IsDocumentOpen Then
        If currentStatesCollection(CStr(States.IsDocumentOpen)) = False Then
            GetDisabledString = superTipDisabledCollection(CStr(States.IsDocumentOpen))
            Exit Function
        End If
    End If

    ' iterate through each document state
    Dim aState
    For Each aState In DocumentStates
        ' use the aState bitmask against the value stored in buttonStates
        ' to determine if the state really applies
        If buttonStates(id) And aState Then
            ' during each loop check the current state value and apply it to
            ' the result. Once the loop completes the IsEnabled value
            ' should reflect if the button/menu item should be enabled
            aCurrentState = currentStatesCollection(CStr(aState))
            If aCurrentState = False Then
                    ' add to the disabled string
                If GetDisabledString <> "" Then
                    GetDisabledString = GetDisabledString & ", " & superTipDisabledCollection(CStr(aState))
                    Else
                    GetDisabledString = superTipDisabledCollection(CStr(aState))
                    End If
                End If
            End If
    Next
    Exit Function
err:
    GetDisabledString = ""
End Function


Private Function IsNotInReadingView() As Boolean
    IsNotInReadingView = Not (ActiveWindow.View.Type = wdReadingView)
End Function

Private Function IsWordXPActiveWindowViewSplitSpecial() As Boolean
    IsWordXPActiveWindowViewSplitSpecial = Not (ActiveDocument.ActiveWindow.View.SplitSpecial = wdPaneRevisions And _
                    ActiveDocument.ActiveWindow.ActivePane.index > 1)
End Function

' called by the MTW5 class to inform the ribbon to update itself
' by initiating call backs to determine enable/disable state of buttons
Public Sub Invalidate()
    If Not IsEmpty(gMTRibbon) Then
        If Not (gMTRibbon Is Nothing) Then
            gMTRibbon.Invalidate
        End If
    End If
End Sub

' called by the MTW5 class to initiate an update of the CommandBars
' (i.e. menu items and buttons) to determine their enable/disable state
Public Sub UpdatePre2007CommandBar()

    WriteLog "UpdatePre2007CommandBar start"

    Dim MTToolbar As CommandBar
    Dim cb As CommandBar
    Dim cbc As CommandBarControl
    Dim foundMenuBar As Boolean
    Dim foundMathType As Boolean
    foundMenuBar = False
    foundMathType = False

    On Error GoTo errHandler
    Dim i As Long
    For i = 1 To CommandBars.count
        Set cb = CommandBars(i)
        If Not (cb Is Nothing) And (cb.name = "Menu Bar" Or cb.name = "MathType") Then

            WriteLog "cb.Name: " & cb.name
            If cb.name = "Menu Bar" Then foundMenuBar = True
            If cb.name = "MathType" Then foundMathType = True

            UpdatePre2007UIItem cb, mtbIDBrowseNext
            UpdatePre2007UIItem cb, mtbIDBrowsePrev
            UpdatePre2007UIItem cb, mtbIDBrowseType
            UpdatePre2007UIItem cb, mtbIDConvertEqns
            UpdatePre2007UIItem cb, mtbIDTeXToggle
            UpdatePre2007UIItem cb, mtbIDEquationReference
            UpdatePre2007UIItem cb, mtbIDExportEqns
            UpdatePre2007UIItem cb, mtbIDFormatEqnNums
            UpdatePre2007UIItem cb, mtbIDFormatEqns
            UpdatePre2007UIItem cb, mtbIDFutureMT
            UpdatePre2007UIItem cb, mtbIDHelp
            UpdatePre2007UIItem cb, mtbIDHelpAboutMT
            UpdatePre2007UIItem cb, mtbIDHelpContents
            UpdatePre2007UIItem cb, mtbIDHelpMTInWord
            UpdatePre2007UIItem cb, mtbIDInsHandEqn

            Set cbc = UpdatePre2007UIItem(cb, mtbIDHelpUnlockReg)
            If Not (cbc Is Nothing) Then
                cbc.caption = RunUICallback("NoDirectCall_GetUnlockUI")
            End If
            UpdatePre2007UIItem cb, mtbIDInsDispMTEqn
            UpdatePre2007UIItem cb, mtbIDInsDispMTEqnLeftNum
            UpdatePre2007UIItem cb, mtbIDInsDispMTEqnRightNum
            UpdatePre2007UIItem cb, mtbIDInsertNextChapter
            UpdatePre2007UIItem cb, mtbIDInsertNextSection
            UpdatePre2007UIItem cb, mtbIDInsertNumber
            UpdatePre2007UIItem cb, mtbIDInsInlineMTEqn
            UpdatePre2007UIItem cb, mtbIDMathPage
            UpdatePre2007UIItem cb, mtbIDModifyBreak
            UpdatePre2007UIItem cb, mtbIDMoreBreaks
            UpdatePre2007UIItem cb, mtbIDOMMathPage
            UpdatePre2007UIItem cb, mtbIDSetEqnPrefs
            UpdatePre2007UIItem cb, mtbIDUpdateEqnNums
            UpdatePre2007UIItem cb, mtbIDWeb
            UpdatePre2007UIItem cb, mtbIDWebEmailFeedback
            UpdatePre2007UIItem cb, mtbIDWebHomePage
            UpdatePre2007UIItem cb, mtbIDWebOrderMT
            UpdatePre2007UIItem cb, mtbIDWebSupport
            UpdatePre2007UIItem cb, mtbIDInsHandEqn
            UpdatePre2007UIItem cb, mtbIDMTOptions
            UpdatePre2007UIItem cb, mtbIDSpeak
        End If
    Next

    WriteLog "  foundMenuBar: " & foundMenuBar
    WriteLog "  foundMathType: " & foundMathType

finalize:
    On Error GoTo abort:
    Dim aTemplate As Template
    For Each aTemplate In Templates
        If aTemplate.name = MacroContainer.name Then
            aTemplate.saved = True
            Exit For
        End If
    Next

    WriteLog "UpdatePre2007CommandBar end"
    Exit Sub

errHandler:
    WriteLog "UpdatePre2007CommandBar error"
    Resume finalize

abort:
    WriteLog "UpdatePre2007CommandBar error"
End Sub

Private Function UpdatePre2007UIItem(cmdBar As CommandBar, id As String) As CommandBarControl

    'WriteLog "  UpdatePre2007UIItem start"
    On Error GoTo errHandler

    Dim cbc As CommandBarControl
    Set cbc = cmdBar.FindControl(id:=1, tag:=id, Recursive:=True)
    Set UpdatePre2007UIItem = cbc
    If Not (cbc Is Nothing) Then
        cbc.enabled = RunUICallback("NoDirectCall_GenEnabledByAppFunctionality", id)
        If (id = mtbIDInsHandEqn) Then
            WriteLog "GenEnabledByAppFunctionality set cbc.enbled to " & cbc.enabled
            ' Disable no matter what if the MIP isn't there
            Dim impa As Boolean
            impa = IsMathInputPanelAvailable
            WriteLog "Call to IsMathInputPanelAvailable returned " & impa
            If Not impa Then cbc.enabled = False
            WriteLog "Final value of cbc.enabled is " & cbc.enabled
            Exit Function
        End If
    End If

    'WriteLog "  UpdatePre2007UIItem end"

    Exit Function

errHandler:
    WriteLog "  UpdatePre2007UIItem error"

End Function

' this subroutine makes sure that the various state collections are populated with data
' returns true if no errors happened, false otherwise
Public Function InitializeStatesCollection() As Boolean
    InitializeStatesCollection = True ' default to success
    On Error GoTo err
    ' setup a collection with nothing more than the values found in the states enumeration
    If DocumentStates Is Nothing Then
        Set DocumentStates = New Collection
        DocumentStates.Add States.AlwaysEnable
        DocumentStates.Add States.IsDocumentOpen
        DocumentStates.Add States.MathPageOK
        DocumentStates.Add States.MTReferenceExists
        DocumentStates.Add States.SelectionInTextBox
        DocumentStates.Add States.NotInReadingView
        DocumentStates.Add States.Word97SelectionInCommentPane
        DocumentStates.Add States.Word97SelectionInFootnoteEndnotePane
        DocumentStates.Add States.Word97SelectionInHeaderFooter
        DocumentStates.Add States.WordXPActiveWindowViewSplitSpecial
        DocumentStates.Add States.IsFunctionalityOK
        DocumentStates.Add States.NotInUnsupportedView
    End If

    ' setup the current and previous states to dummy values if needed
    If currentStatesCollection Is Nothing Then
        Set currentStatesCollection = New Collection
        currentStatesCollection.Add False, CStr(States.AlwaysEnable)
        currentStatesCollection.Add False, CStr(States.IsDocumentOpen)
        currentStatesCollection.Add False, CStr(States.MathPageOK)
        currentStatesCollection.Add False, CStr(States.MTReferenceExists)
        currentStatesCollection.Add False, CStr(States.SelectionInTextBox)
        currentStatesCollection.Add False, CStr(States.NotInReadingView)
        currentStatesCollection.Add False, CStr(States.Word97SelectionInCommentPane)
        currentStatesCollection.Add False, CStr(States.Word97SelectionInFootnoteEndnotePane)
        currentStatesCollection.Add False, CStr(States.Word97SelectionInHeaderFooter)
        currentStatesCollection.Add False, CStr(States.WordXPActiveWindowViewSplitSpecial)
        currentStatesCollection.Add False, CStr(States.IsFunctionalityOK)
        currentStatesCollection.Add False, CStr(States.NotInUnsupportedView)
    End If
    If previousStatesCollection Is Nothing Then
        Set previousStatesCollection = New Collection
        previousStatesCollection.Add False, CStr(States.AlwaysEnable)
        previousStatesCollection.Add False, CStr(States.IsDocumentOpen)
        previousStatesCollection.Add False, CStr(States.MathPageOK)
        previousStatesCollection.Add False, CStr(States.MTReferenceExists)
        previousStatesCollection.Add False, CStr(States.SelectionInTextBox)
        previousStatesCollection.Add False, CStr(States.NotInReadingView)
        previousStatesCollection.Add False, CStr(States.Word97SelectionInCommentPane)
        previousStatesCollection.Add False, CStr(States.Word97SelectionInFootnoteEndnotePane)
        previousStatesCollection.Add False, CStr(States.Word97SelectionInHeaderFooter)
        previousStatesCollection.Add False, CStr(States.WordXPActiveWindowViewSplitSpecial)
        previousStatesCollection.Add False, CStr(States.IsFunctionalityOK)
        previousStatesCollection.Add False, CStr(States.NotInUnsupportedView)
    End If

    If superTipDisabledCollection Is Nothing Then
        Set superTipDisabledCollection = New Collection

        superTipDisabledCollection.Add "", CStr(States.AlwaysEnable) ' special case
        superTipDisabledCollection.Add UILib.GetUserString("!3001a document is not open"), CStr(States.IsDocumentOpen)
        superTipDisabledCollection.Add UILib.GetUserString("!3010the document has not been saved"), CStr(States.MathPageOK)
        superTipDisabledCollection.Add UILib.GetUserString("!3002a reference has not been added"), CStr(States.MTReferenceExists)
        superTipDisabledCollection.Add UILib.GetUserString("!3003the current selection is inside a textbox"), CStr(States.SelectionInTextBox)
        superTipDisabledCollection.Add UILib.GetUserString("!3004the application is in Reading View"), CStr(States.NotInReadingView)
        superTipDisabledCollection.Add UILib.GetUserString("!3005the current selection is inside a comment pane"), CStr(States.Word97SelectionInCommentPane)
        superTipDisabledCollection.Add UILib.GetUserString("!3006the current selection is inside a footnote or endnote pane"), CStr(States.Word97SelectionInFootnoteEndnotePane)
        superTipDisabledCollection.Add UILib.GetUserString("!3007the current selection is inside a header or footer"), CStr(States.Word97SelectionInHeaderFooter)
        superTipDisabledCollection.Add UILib.GetUserString("!3008the document has a split window"), CStr(States.WordXPActiveWindowViewSplitSpecial)
        superTipDisabledCollection.Add UILib.GetUserString("!3009the MathType 30-day demo has expired"), CStr(States.IsFunctionalityOK)
        superTipDisabledCollection.Add UILib.GetUserString("!3016the document is in an unsupported view"), CStr(States.NotInUnsupportedView)
    End If

    If superTipEnabledCollection Is Nothing Then
        Set superTipEnabledCollection = New Collection
        superTipEnabledCollection.Add UILib.GetUserString("!3041Insert an inline MathType equation."), mtbIDInsInlineMTEqn
        superTipEnabledCollection.Add UILib.GetUserString("!3042Insert a centered MathType display equation."), mtbIDInsDispMTEqn
        superTipEnabledCollection.Add UILib.GetUserString("!3043Insert display MathType equation with number on left."), mtbIDInsDispMTEqnLeftNum
        superTipEnabledCollection.Add UILib.GetUserString("!3044Insert display MathType equation with number on right."), mtbIDInsDispMTEqnRightNum
        superTipEnabledCollection.Add UILib.GetUserString("!3043Insert display MathType equation with number on left."), mtbIDInsDispMTEqnLeftNum2
        superTipEnabledCollection.Add UILib.GetUserString("!3044Insert display MathType equation with number on right."), mtbIDInsDispMTEqnRightNum2

        superTipEnabledCollection.Add UILib.GetUserString("!3041Insert an inline MathType equation."), mtbIDInsInlineMTEqn3
        superTipEnabledCollection.Add UILib.GetUserString("!3042Insert a centered MathType display equation."), mtbIDInsDispMTEqn3
        superTipEnabledCollection.Add UILib.GetUserString("!3047Insert a MathType equation from handwriting"), mtbIDInsHandEqn
        superTipEnabledCollection.Add UILib.GetUserString("!3043Insert display MathType equation with number on left."), mtbIDInsDispMTEqnLeftNum3
        superTipEnabledCollection.Add UILib.GetUserString("!3044Insert display MathType equation with number on right."), mtbIDInsDispMTEqnRightNum3

        superTipEnabledCollection.Add UILib.GetUserString("!3045Insert display Equation Builder equation with number on left."), mtbIDInsEBEqnLeftNum
        superTipEnabledCollection.Add UILib.GetUserString("!3046Insert display Equation Builder equation with number on right."), mtbIDInsEBEqnRightNum
        superTipEnabledCollection.Add UILib.GetUserString("!3050Insert an equation number at the insertion point."), mtbIDInsertNumber
        superTipEnabledCollection.Add UILib.GetUserString("!3051Set the format of equation numbers."), mtbIDFormatEqnNums
        superTipEnabledCollection.Add UILib.GetUserString("!3052Update equation numbers after rearranging the document."), mtbIDUpdateEqnNums
        superTipEnabledCollection.Add UILib.GetUserString("!3079Insert a reference to an existing equation."), mtbIDEquationReference
        superTipEnabledCollection.Add UILib.GetUserString("!3080Manage chapter and section breaks for equation numbering."), mtbIDManageChapterSections
        superTipEnabledCollection.Add UILib.GetUserString("!3053Starts a new equation numbering section."), mtbIDInsertNextSection
        superTipEnabledCollection.Add UILib.GetUserString("!3054Starts a new equation numbering chapter."), mtbIDInsertNextChapter
        superTipEnabledCollection.Add UILib.GetUserString("!3055Insert a break with a specified starting number."), mtbIDMoreBreaks
        superTipEnabledCollection.Add UILib.GetUserString("!3056Modify an existing chapter or section break."), mtbIDModifyBreak
        superTipEnabledCollection.Add UILib.GetUserString("!3057Browse equations, equation numbers, chapter and section breaks."), mtbIDBrowseType
        superTipEnabledCollection.Add UILib.GetUserString("!3058Loads fonts, styles and sizes from MathType preference files."), mtbIDSetEqnPrefs
        superTipEnabledCollection.Add UILib.GetUserString("!3059Format some or all of the MathType equations in this document."), mtbIDFormatEqns
        superTipEnabledCollection.Add UILib.GetUserString("!3060Convert equations in a document to TeX or MathML."), mtbIDConvertEqns
        superTipEnabledCollection.Add UILib.GetUserString("!3081Convert a run of TeX/LaTeX equation language to a MathType equation or vice-versa. Enclose TeX in $...$ for an inline equation, \[...\] for a display (paragraph) equation."), mtbIDTeXToggle
        superTipEnabledCollection.Add UILib.GetUserString("!3061Export equations to EPS or GIF image files."), mtbIDExportEqns
        superTipEnabledCollection.Add UILib.GetUserString("!3062Publish the document to a math-savvy web page."), mtbIDMathPage
        superTipEnabledCollection.Add UILib.GetUserString("!3063Allows you to enter a MathType product key and gain access to full MathType functionality."), mtbIDHelpUnlockReg
        superTipEnabledCollection.Add UILib.GetUserString("!3064View MathType's version number and your product key."), mtbIDHelpAboutMT
        superTipEnabledCollection.Add UILib.GetUserString("!3065Open MathType's home page in your web browser."), mtbIDWeb
        superTipEnabledCollection.Add UILib.GetUserString("!3065Open MathType's home page in your web browser."), mtbIDWebHomePage
        superTipEnabledCollection.Add UILib.GetUserString("!3077Go to MathType's online support area."), mtbIDWebSupport
        superTipEnabledCollection.Add UILib.GetUserString("!3078Prepare an email ready to send feedback to Design Science."), mtbIDWebEmailFeedback
        superTipEnabledCollection.Add "", mtbIDBrowsePrev
        superTipEnabledCollection.Add "", mtbIDBrowseNext
        superTipEnabledCollection.Add "", mtbIDHelp
        superTipEnabledCollection.Add "", mtbIDHelpContents
        superTipEnabledCollection.Add "", mtbIDHelpContentsMac
        superTipEnabledCollection.Add "", mtbIDHelpMTInWord
        superTipEnabledCollection.Add "", mtbIDHelpMTInWordMac
        superTipEnabledCollection.Add "", mtbIDWebOrderMT
        superTipEnabledCollection.Add "", mtbIDWebOrderMTMac
        superTipEnabledCollection.Add "", mtbIDFutureMT
        superTipEnabledCollection.Add UILib.GetUserString2("3305", "3314", "MathType options for Word"), mtbIDMTOptions
    End If

    ' the rest of this function sets up which button ID is associated with a
    ' specific enable/disable state

    ' this value is used to hold the state bits. It is or'd together with
    ' one or more of the states enumeration values. A set of buttons may share
    ' a particular set of bits, so this value can be used multiple times.
    ' Because this variable is reset for each new state, you must be careful
    ' when editing the code.
    Dim state As Long

    If buttonStates Is Nothing Then
        Set buttonStates = New Collection
    End If

    If buttonStates.count = 0 Then
        '**** start of new state ****
        state = States.IsDocumentOpen
        state = state Or States.NotInReadingView
        state = state Or States.NotInUnsupportedView

        buttonStates.Add state, mtbIDInsInlineMTEqn
        buttonStates.Add state, mtbIDInsInlineMTEqn3
        buttonStates.Add state, mtbIDInsInlineMTEqnMac

        ' now add the lite mode check
        state = state Or States.IsFunctionalityOK

        buttonStates.Add state, mtbIDFormatEqnNums ' works in full/eval mode only
        buttonStates.Add state, mtbIDUpdateEqnNums ' works in full/eval mode only
        buttonStates.Add state, mtbIDEquationReference ' works in full/eval mode only
        buttonStates.Add state, mtbIDSetEqnPrefs ' works in full/eval mode only
        buttonStates.Add state, mtbIDFormatEqns ' works in full/eval mode only
        buttonStates.Add state, mtbIDConvertEqns ' works in full/eval mode only
        buttonStates.Add state, mtbIDExportEqns ' works in full/eval mode only
        buttonStates.Add state, mtbIDTeXToggle ' works in full/eval mode only
        buttonStates.Add state, mtbIDInsHandEqn 'FIX

        '**** start of new state ****
        state = States.IsDocumentOpen Or States.Word97SelectionInCommentPane Or _
                States.Word97SelectionInHeaderFooter Or _
                States.Word97SelectionInFootnoteEndnotePane Or States.SelectionInTextBox

        If Val(Application.version) = kWordX Then
            state = state Or States.WordXPActiveWindowViewSplitSpecial
        End If

        state = state Or States.NotInReadingView
        state = state Or States.NotInUnsupportedView

        buttonStates.Add state, mtbIDInsDispMTEqn
        buttonStates.Add state, mtbIDInsDispMTEqn3
        buttonStates.Add state, mtbIDInsDispMTEqnMac

        ' now add the lite mode check
        state = state Or States.IsFunctionalityOK

        buttonStates.Add state, mtbIDInsDispMTEqnLeftNum ' works in full/eval mode only
        buttonStates.Add state, mtbIDInsDispMTEqnRightNum ' works in full/eval mode only
        buttonStates.Add state, mtbIDInsDispMTEqnLeftNum2 ' works in full/eval mode only
        buttonStates.Add state, mtbIDInsDispMTEqnRightNum2 ' works in full/eval mode only
        buttonStates.Add state, mtbIDInsDispMTEqnLeftNum3 ' works in full/eval mode only
        buttonStates.Add state, mtbIDInsDispMTEqnLeftNumMac ' works in full/eval mode only
        buttonStates.Add state, mtbIDInsDispMTEqnRightNum3 ' works in full/eval mode only
        buttonStates.Add state, mtbIDInsDispMTEqnRightNumMac ' works in full/eval mode only
        buttonStates.Add state, mtbIDInsertNumber ' works in full/eval mode only
        buttonStates.Add state, mtbIDManageChapterSections ' works in full/eval mode only
        buttonStates.Add state, mtbIDInsertNextSection ' works in full/eval mode only
        buttonStates.Add state, mtbIDInsertNextChapter ' works in full/eval mode only
        buttonStates.Add state, mtbIDMoreBreaks ' works in full/eval mode only
        buttonStates.Add state, mtbIDModifyBreak ' works in full/eval mode only
        buttonStates.Add state, mtbIDMTOptions ' works in full/eval mode only
        buttonStates.Add state, mtbIDMTOptionsMac ' works in full/eval mode only

        '**** start of new state ****
        state = States.IsDocumentOpen
        state = state Or States.NotInUnsupportedView

        buttonStates.Add state, mtbIDBrowsePrev
        buttonStates.Add state, mtbIDBrowseType
        buttonStates.Add state, mtbIDBrowseNext

        ' I removed the dependency on States.MathPageOK, but left the machinery to
        ' maintain and update States.MathPageOK for use it in the future. See MT-937 -RM
        ' state = States.IsDocumentOpen Or States.MathPageOK Or States.IsFunctionalityOK
        state = state Or States.IsFunctionalityOK

        buttonStates.Add state, mtbIDMathPage ' works in full/eval mode only
        buttonStates.Add state, mtbIDOMMathPage ' works in full/eval mode only

        '**** start of new state ****
        state = States.AlwaysEnable

        buttonStates.Add state, mtbIDHelp
        buttonStates.Add state, mtbIDHelpContents
        buttonStates.Add state, mtbIDHelpContentsMac
        buttonStates.Add state, mtbIDHelpMTInWord
        buttonStates.Add state, mtbIDHelpMTInWordMac
        buttonStates.Add state, mtbIDHelpUnlockReg
        buttonStates.Add state, mtbIDHelpUnlockRegMac
        buttonStates.Add state, mtbIDHelpAboutMT
        buttonStates.Add state, mtbIDHelpAboutMTMac

        buttonStates.Add state, mtbIDWeb
        buttonStates.Add state, mtbIDWebHomePage
        buttonStates.Add state, mtbIDWebSupport
        buttonStates.Add state, mtbIDWebSupportMac
        buttonStates.Add state, mtbIDWebEmailFeedback
        buttonStates.Add state, mtbIDWebEmailFeedbackMac
        buttonStates.Add state, mtbIDWebOrderMT
        buttonStates.Add state, mtbIDWebOrderMTMac
        buttonStates.Add state, mtbIDFutureMT
        buttonStates.Add state, mtbIDFutureMTMac
        buttonStates.Add state, mtbIDSpeak
        buttonStates.Add state, mtbIDSpeak3

        'state = state Or States.IsFunctionalityOK
        'buttonStates.Add state, mtbIDMTOptions

    End If
    Exit Function
err:
    InitializeStatesCollection = False
End Function

' Since the individual items stored within a VB collection can not be updated
' this subroutine will do the dirty work of manipulating the collection
' to update a given value that is associated with a given key.
Private Sub UpdateCollection(col As Collection, key As Long, value)
    Dim keyStr As String
    Dim tmp

    ' our keys are really long values, but VB does not accept that
    ' so convert to a string
    keyStr = CStr(key)
    ' check if key exists by using the item command to retreive the
    ' value associated with a key. If the item does not exist, we get
    ' an error, and jump to "keymissing"
    On Error GoTo keymissing
    tmp = col.item(keyStr) ' could fail

    col.Remove keyStr ' could fail but probably won't if it got this far
    col.Add value, keyStr
    GoTo done
keymissing:
    col.Add value, keyStr
    GoTo done
done:
End Sub

Public Sub UIUpdate()
    WriteLog "UIUpdate start"
    ' refresh state
    If UpdateState Then
        ' if needed, update UI elements
        #If Win32 Then
        If Val(Application.version) >= kWord2007 Then
            Invalidate
        Else
            UpdatePre2007CommandBar
        End If
        #Else
        UpdatePre2007CommandBar
        #End If
    End If
    WriteLog "UIUpdate end"
End Sub
