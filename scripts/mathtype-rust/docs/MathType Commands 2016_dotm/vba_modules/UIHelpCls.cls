Attribute VB_Name = "UIHelpCls"
Attribute VB_Base = "0{FCFB3D2A-A0FA-1068-A738-08002B3371B5}"
Attribute VB_GlobalNameSpace = False
Attribute VB_Creatable = False
Attribute VB_PredeclaredId = False
Attribute VB_Exposed = False
Attribute VB_TemplateDerived = False
Attribute VB_Customizable = False

'=====================================================================
' (c) Copyright 2007-2010 by Design Science, Inc. All rights reserved.
'$Header: /MathType/Windows/WordMacros/UIHelpCls.cls 16    10/11/11 2:12p Jimm $
'=====================================================================

' Expose the error description
' This error number is required because the CallByName
' function does not return the correct error
Public Property Get ErrorDescription() As String
    ErrorDescription = m_sError
End Property

' Expose the error number
' This error number is required because the CallByName
' function does not return the correct error
Public Property Get ErrorNumber() As Long
    ErrorNumber = m_lError
End Property

Private Sub OpenUrl(url As String)
    ShellExecute 0, "Open", url
End Sub

Public Sub NoDirectCall_ShowHelpContents()
    OpenUrl "https://docs.wiris.com/en/mathtype/mathtype_desktop/start?utm_source=Product&utm_medium=MathTypeWin"
    MTIncrementStatisticBy "HCts", 1
End Sub

Public Sub NoDirectCall_ShowHelpMTInWord()
    OpenUrl "https://docs.wiris.com/en/mathtype/mathtype_desktop/microsoft_office?utm_source=Product&utm_medium=MathTypeWin"
    MTIncrementStatisticBy "HWrd", 1
End Sub

Public Sub NoDirectCall_ShowUnlockReg()
    OpenUrl "http://docs.wiris.com/en/mathtype/mathtype_desktop/installation?utm_source=Product&utm_medium=MathTypeWin#activate_the_product"
End Sub

Public Sub NoDirectCall_ShowWebHomePage()
    OpenUrl "http://docs.wiris.com/en/mathtype/start?utm_source=Product&utm_medium=MathTypeWin"
End Sub

Public Sub NoDirectCall_ShowWebSupport()
    OpenUrl "http://www.wiris.com/en/contact?utm_source=Product&utm_medium=MathTypeWin"
End Sub

Public Sub NoDirectCall_ShowWebOrderMathType()
    OpenUrl "https://store.wiris.com/en/purchase/step1?utm_source=Product&utm_medium=MathTypeWin"
End Sub

Public Sub NoDirectCall_ShowFutureMT()
    OpenUrl "http://www.wiris.com/en/contact?utm_source=Product&utm_medium=MathTypeWin"
End Sub

Public Sub NoDirectCall_ShowWebEmailFeedback()
    If Application.name = kAppMSPP Then
        OpenUrl "http://www.wiris.com/en/contact?utm_source=Product&utm_medium=MathTypeWin"
    ElseIf Application.name = kAppMSW Then
        OpenUrl "http://www.wiris.com/en/contact?utm_source=Product&utm_medium=MathTypeWin"
    End If
End Sub

Public Sub NoDirectCall_ShowAboutMT()
    Dim result As Long
    result = MTShowAboutBox
End Sub
