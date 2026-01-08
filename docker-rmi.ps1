# docker-rmi.ps1
<#
.SYNOPSIS
    清理 Docker 镜像，根据版本号时间删除过期镜像

.DESCRIPTION
    此脚本根据镜像 Tag 中的版本号来识别过期镜像，并根据指定的删除模式进行清理。
    支持的版本号格式：
    - vYYMMDDHHMMSS (12位，如 v251208120898)
    - vYYMMDDHHMM (10位，如 v2512081208)
    - vYYMMDD (6位，如 v251208，默认时间为 00:00:00)
    
    支持预览模式、强制删除和多种删除策略。

.PARAMETER ImageFilter
    指定要清理的镜像名称过滤条件，例如: kt-vue, nginx, 或 *
    此参数为必需参数。

.PARAMETER BeforeTime
    指定删除时间之前的镜像，格式: YYYY-MM-DD HH:MM:SS 或 YYYY-MM-DD
    此参数为必需参数。

.PARAMETER Preview
    预览模式，不实际删除镜像，只显示将要删除的镜像列表
    此参数为可选参数。

.PARAMETER Force
    强制删除，跳过确认提示
    此参数为可选参数。

.PARAMETER DeleteMode
    指定删除模式，可选值：Auto（默认）、Force、TagOnly
    - Auto: 单 Tag 正常删除，多 Tag 强制删除
    - Force: 所有镜像都使用 -f 强制删除
    - TagOnly: 只删除过期的 tag，保留镜像本身
    此参数为可选参数。

.EXAMPLE
    预览将要删除的镜像
    PS C:\> .\docker-rmi.ps1 -ImageFilter "kt-vue" -BeforeTime "2025-12-09" -Preview

.EXAMPLE
    自动模式删除过期镜像
    PS C:\> .\docker-rmi.ps1 -ImageFilter "kt-vue" -BeforeTime "2025-12-09" -DeleteMode Auto

.EXAMPLE
    强制删除所有过期镜像（跳过确认）
    PS C:\> .\docker-rmi.ps1 -ImageFilter "kt-vue" -BeforeTime "2025-12-09" -DeleteMode Force -Force

.EXAMPLE
    只删除过期的 tag，保留镜像
    PS C:\> .\docker-rmi.ps1 -ImageFilter "kt-vue" -BeforeTime "2025-12-09" -DeleteMode TagOnly

.EXAMPLE
    删除 30 天前的所有镜像
    PS C:\> $targetDate = (Get-Date).AddDays(-30).ToString("yyyy-MM-dd")
    PS C:\> .\docker-rmi.ps1 -ImageFilter "*" -BeforeTime $targetDate -Preview

.NOTES
    文件名: docker-rmi.ps1
    作者: AI Assistant
    创建日期: 2025-01-08
    版本: 2.0

.LINK
    https://docs.docker.com/engine/reference/commandline/rmi/
#>

param(
    [Parameter(Mandatory=$false)]
    [switch]$Help = $false,
    
    [Parameter(Mandatory=$false, HelpMessage="指定要清理的镜像名称过滤条件，例如: kt-vue, nginx, 或 *")]
    [string]$ImageFilter,
    
    [Parameter(Mandatory=$false, HelpMessage="指定删除时间之前的镜像，格式: YYYY-MM-DD HH:MM:SS 或 YYYY-MM-DD")]
    [string]$BeforeTime,
    
    [Parameter(Mandatory=$false)]
    [switch]$Preview = $false,
    
    [Parameter(Mandatory=$false)]
    [switch]$Force = $false,
    
    [Parameter(Mandatory=$false)]
    [ValidateSet("Auto", "Force", "TagOnly")]
    [string]$DeleteMode = "Auto"
)

# 显示完整使用说明的函数
function Show-Usage {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "Docker 镜像清理工具" -ForegroundColor Cyan
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "" -ForegroundColor White
    Write-Host "用法:" -ForegroundColor Yellow
    Write-Host "  .\docker-rmi.ps1 -ImageFilter <镜像过滤条件> -BeforeTime <删除时间> [选项]" -ForegroundColor White
    Write-Host "" -ForegroundColor White
    Write-Host "必需参数:" -ForegroundColor Yellow
    Write-Host "  -ImageFilter <字符串>" -ForegroundColor White
    Write-Host "      指定要清理的镜像名称过滤条件，例如: kt-vue, nginx, 或 *" -ForegroundColor Gray
    Write-Host "      支持通配符，如 * 匹配所有镜像" -ForegroundColor Gray
    Write-Host "" -ForegroundColor White
    Write-Host "  -BeforeTime <日期时间>" -ForegroundColor White
    Write-Host "      指定删除时间之前的镜像" -ForegroundColor Gray
    Write-Host "      格式: YYYY-MM-DD HH:MM:SS 或 YYYY-MM-DD" -ForegroundColor Gray
    Write-Host "      例如: "2025-12-09" 或 "2025-12-09 14:30:00"" -ForegroundColor Gray
    Write-Host "" -ForegroundColor White
    Write-Host "可选参数:" -ForegroundColor Yellow
    Write-Host "  -Preview" -ForegroundColor White
    Write-Host "      预览模式，不实际删除镜像，只显示将要删除的镜像列表" -ForegroundColor Gray
    Write-Host "      建议先使用此参数查看将要删除的镜像" -ForegroundColor Gray
    Write-Host "" -ForegroundColor White
    Write-Host "  -Force" -ForegroundColor White
    Write-Host "      强制删除，跳过确认提示" -ForegroundColor Gray
    Write-Host "      使用此参数时不会询问确认" -ForegroundColor Gray
    Write-Host "" -ForegroundColor White
    Write-Host "  -DeleteMode <模式>" -ForegroundColor White
    Write-Host "      指定删除模式，可选值: Auto, Force, TagOnly" -ForegroundColor Gray
    Write-Host "      Auto (默认): 单 Tag 正常删除，多 Tag 强制删除" -ForegroundColor Gray
    Write-Host "      Force: 所有镜像都使用 -f 强制删除" -ForegroundColor Gray
    Write-Host "      TagOnly: 只删除过期的 tag，保留镜像本身" -ForegroundColor Gray
    Write-Host "" -ForegroundColor White
    Write-Host "支持的版本号格式:" -ForegroundColor Yellow
    Write-Host "  vYYMMDDHHMMSS (12位)" -ForegroundColor White
    Write-Host "      例如: v251208120898 代表 2025-12-08 12:08:98" -ForegroundColor Gray
    Write-Host "" -ForegroundColor White
    Write-Host "  vYYMMDDHHMM (10位)" -ForegroundColor White
    Write-Host "      例如: v2512081208 代表 2025-12-08 12:08:00" -ForegroundColor Gray
    Write-Host "" -ForegroundColor White
    Write-Host "  vYYMMDD (6位)" -ForegroundColor White
    Write-Host "      例如: v251208 代表 2025-12-08 00:00:00" -ForegroundColor Gray
    Write-Host "" -ForegroundColor White
    Write-Host "使用示例:" -ForegroundColor Yellow
    Write-Host "  # 预览将要删除的镜像" -ForegroundColor Gray
    Write-Host "  .\docker-rmi.ps1 -ImageFilter "kt-vue" -BeforeTime "2025-12-09" -Preview" -ForegroundColor White
    Write-Host "" -ForegroundColor White
    Write-Host "  # 自动模式删除过期镜像" -ForegroundColor Gray
    Write-Host "  .\docker-rmi.ps1 -ImageFilter "kt-vue" -BeforeTime "2025-12-09" -DeleteMode Auto" -ForegroundColor White
    Write-Host "" -ForegroundColor White
    Write-Host "  # 强制删除所有过期镜像（跳过确认）" -ForegroundColor Gray
    Write-Host "  .\docker-rmi.ps1 -ImageFilter "kt-vue" -BeforeTime "2025-12-09" -DeleteMode Force -Force" -ForegroundColor White
    Write-Host "" -ForegroundColor White
    Write-Host "  # 只删除过期的 tag，保留镜像" -ForegroundColor Gray
    Write-Host "  .\docker-rmi.ps1 -ImageFilter "kt-vue" -BeforeTime "2025-12-09" -DeleteMode TagOnly" -ForegroundColor White
    Write-Host "" -ForegroundColor White
    Write-Host "  # 删除 30 天前的所有镜像" -ForegroundColor Gray
    Write-Host "  `$targetDate = (Get-Date).AddDays(-30).ToString("yyyy-MM-dd")" -ForegroundColor White
    Write-Host "  .\docker-rmi.ps1 -ImageFilter "*" -BeforeTime `$targetDate -Preview" -ForegroundColor White
    Write-Host "" -ForegroundColor White
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "" -ForegroundColor White
}

# 检查是否显示帮助
if ($Help) {
    Show-Usage
    exit 0
}

# 检查是否提供了必需参数
if (-not $PSBoundParameters.ContainsKey('ImageFilter') -or -not $PSBoundParameters.ContainsKey('BeforeTime')) {
    Write-Host "请提供必需参数..." -ForegroundColor Yellow
    Write-Host "提示: 输入 'help 或 ? 或 -h' 查看参数帮助信息" -ForegroundColor Gray
    Write-Host "" -ForegroundColor White
    
    # 交互式输入 ImageFilter
    if ([string]::IsNullOrEmpty($ImageFilter)) {
        do {
            $ImageFilter = Read-Host "ImageFilter (例如: kt-vue, nginx, 或 *)"
            if ($ImageFilter -eq "help" -or $ImageFilter -eq "?" -or $ImageFilter -eq "-h") {
                Write-Host "" -ForegroundColor White
                Write-Host "ImageFilter 参数说明:" -ForegroundColor Cyan
                Write-Host "  - 指定要清理的镜像名称过滤条件" -ForegroundColor Gray
                Write-Host "  - 支持通配符，如 * 匹配所有镜像" -ForegroundColor Gray
                Write-Host "  - 示例: kt-vue, nginx, test-*" -ForegroundColor Gray
                Write-Host "" -ForegroundColor White
                $ImageFilter = $null
            } elseif ([string]::IsNullOrEmpty($ImageFilter)) {
                Write-Host "错误: ImageFilter 不能为空" -ForegroundColor Red
            }
        } while ([string]::IsNullOrEmpty($ImageFilter))
    }
    
    # 交互式输入 BeforeTime
    if ([string]::IsNullOrEmpty($BeforeTime)) {
        do {
            $BeforeTime = Read-Host "BeforeTime (格式: YYYY-MM-DD 或 YYYY-MM-DD HH:MM:SS)"
            if ($BeforeTime -eq "help" -or $BeforeTime -eq "?" -or $BeforeTime -eq "-h") {
                Write-Host "" -ForegroundColor White
                Write-Host "BeforeTime 参数说明:" -ForegroundColor Cyan
                Write-Host "  - 指定删除时间之前的镜像" -ForegroundColor Gray
                Write-Host "  - 格式: YYYY-MM-DD 或 YYYY-MM-DD HH:MM:SS" -ForegroundColor Gray
                Write-Host "  - 示例: 2025-12-09 或 2025-12-09 14:30:00" -ForegroundColor Gray
                Write-Host "" -ForegroundColor White
                $BeforeTime = $null
            } elseif ([string]::IsNullOrEmpty($BeforeTime)) {
                Write-Host "错误: BeforeTime 不能为空" -ForegroundColor Red
            }
        } while ([string]::IsNullOrEmpty($BeforeTime))
    }
    
    Write-Host "" -ForegroundColor White
}

# 显示使用说明
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "当前配置:" -ForegroundColor Yellow
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  ImageFilter: $ImageFilter" -ForegroundColor White
Write-Host "  BeforeTime: $BeforeTime" -ForegroundColor White
Write-Host "  DeleteMode: $DeleteMode" -ForegroundColor White
Write-Host "  Preview: $Preview" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "" -ForegroundColor White

# 解析指定时间
try {
    $targetTime = [DateTime]::Parse($BeforeTime)
    Write-Host "将删除 $targetTime 之前的镜像" -ForegroundColor Yellow
}
catch {
    Write-Host "时间格式错误: '$BeforeTime'" -ForegroundColor Red
    Write-Host "请使用以下格式之一:" -ForegroundColor Yellow
    Write-Host "  YYYY-MM-DD (例如: 2025-12-09)" -ForegroundColor Gray
    Write-Host "  YYYY-MM-DD HH:MM:SS (例如: 2025-12-09 14:30:00)" -ForegroundColor Gray
    exit 1
}

Write-Host "镜像过滤条件: $ImageFilter" -ForegroundColor Cyan

# 获取所有镜像及其 tag 数量
Write-Host ""
Write-Host "正在分析镜像..." -ForegroundColor Cyan

# 获取所有镜像的完整信息
$imageData = docker images --format "json" | ConvertFrom-Json | Where-Object { $_.Repository -like "*$ImageFilter*" -or $_.Tag -like "*$ImageFilter*" }

if (-not $imageData) {
    Write-Host "未找到匹配 '$ImageFilter' 的镜像" -ForegroundColor Yellow
    exit 0
}

# 按镜像 ID 分组
$imageGroups = @{}
foreach ($img in $imageData) {
    $id = $img.ID
    if (-not $imageGroups.ContainsKey($id)) {
        $imageGroups[$id] = @{
            Images = @()
            ID = $id
        }
    }
    $imageGroups[$id].Images += $img
}

# 解析版本号并筛选过期镜像
$expiredImages = @{}

foreach ($groupId in $imageGroups.Keys) {
    $group = $imageGroups[$groupId]
    
    foreach ($img in $group.Images) {
        $tag = $img.Tag
        
        # 解析版本号，支持多种格式
        # 格式1: vYYMMDDHHMMSS (12位)
        if ($tag -match '^v(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})$') {
            $year = 2000 + [int]$matches[1]
            $month = [int]$matches[2]
            $day = [int]$matches[3]
            $hour = [int]$matches[4]
            $minute = [int]$matches[5]
            $second = [int]$matches[6]
            
            try {
                $versionTime = New-Object DateTime($year, $month, $day, $hour, $minute, $second)
                
                if ($versionTime -lt $targetTime) {
                    if (-not $expiredImages.ContainsKey($groupId)) {
                        $expiredImages[$groupId] = @{
                            Images = @()
                            VersionTime = $versionTime
                            ID = $groupId
                            TagCount = $group.Images.Count
                        }
                    }
                    $expiredImages[$groupId].Images += $img
                }
            }
            catch {
                # 忽略解析错误
            }
        }
        # 格式2: vYYMMDDHHMM (10位)
        elseif ($tag -match '^v(\d{2})(\d{2})(\d{2})(\d{2})(\d{2})$') {
            $year = 2000 + [int]$matches[1]
            $month = [int]$matches[2]
            $day = [int]$matches[3]
            $hour = [int]$matches[4]
            $minute = [int]$matches[5]
            $second = 0
            
            try {
                $versionTime = New-Object DateTime($year, $month, $day, $hour, $minute, $second)
                
                if ($versionTime -lt $targetTime) {
                    if (-not $expiredImages.ContainsKey($groupId)) {
                        $expiredImages[$groupId] = @{
                            Images = @()
                            VersionTime = $versionTime
                            ID = $groupId
                            TagCount = $group.Images.Count
                        }
                    }
                    $expiredImages[$groupId].Images += $img
                }
            }
            catch {
                # 忽略解析错误
            }
        }
        # 格式3: vYYMMDD (6位，默认时间为 00:00:00)
        elseif ($tag -match '^v(\d{2})(\d{2})(\d{2})$') {
            $year = 2000 + [int]$matches[1]
            $month = [int]$matches[2]
            $day = [int]$matches[3]
            $hour = 0
            $minute = 0
            $second = 0
            
            try {
                $versionTime = New-Object DateTime($year, $month, $day, $hour, $minute, $second)
                
                if ($versionTime -lt $targetTime) {
                    if (-not $expiredImages.ContainsKey($groupId)) {
                        $expiredImages[$groupId] = @{
                            Images = @()
                            VersionTime = $versionTime
                            ID = $groupId
                            TagCount = $group.Images.Count
                        }
                    }
                    $expiredImages[$groupId].Images += $img
                }
            }
            catch {
                # 忽略解析错误
            }
        }
        else {
            # 不匹配任何格式，跳过
            continue
        }
    }
}

# 统计
$totalExpired = $expiredImages.Count
$multiTagExpired = ($expiredImages.Values | Where-Object { $_.TagCount -gt 1 }).Count

# 显示报告
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "镜像清理报告" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "BeforeTime: $targetTime" -ForegroundColor White
Write-Host "ImageFilter: $ImageFilter" -ForegroundColor White
Write-Host "过期镜像数: $totalExpired" -ForegroundColor Red
Write-Host "多 Tag 镜像: $multiTagExpired" -ForegroundColor Yellow
Write-Host "DeleteMode: $DeleteMode" -ForegroundColor White
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "" -ForegroundColor White

# 显示详细列表
if ($totalExpired -gt 0) {
    Write-Host "过期镜像详情:" -ForegroundColor Yellow
    Write-Host "----------------------------------------" -ForegroundColor Gray
    
    foreach ($expired in $expiredImages.Values | Sort-Object VersionTime) {
        Write-Host ""
        Write-Host "镜像 ID: $($expired.ID)" -ForegroundColor Cyan
        $versionTimeString = $expired.VersionTime.ToString('yyyy-MM-dd HH:mm:ss')
        Write-Host "版本时间: $versionTimeString" -ForegroundColor White
        Write-Host "总 Tag 数: $($expired.TagCount)" -ForegroundColor Gray
        Write-Host "过期 Tags ($($expired.Images.Count)):" -ForegroundColor Gray
        
        foreach ($img in $expired.Images) {
            $fullName = "$($img.Repository):$($img.Tag)"
            Write-Host "  - $fullName" -ForegroundColor Red
        }
        
        if ($expired.TagCount -gt 1) {
            Write-Host "  [多 Tag 镜像]" -ForegroundColor Yellow
        }
    }
    
    Write-Host ""
    Write-Host "----------------------------------------" -ForegroundColor Gray
}

# 预览模式
if ($Preview) {
    Write-Host ""
    Write-Host "[预览模式] 不会实际删除镜像" -ForegroundColor Yellow
    exit 0
}

# 确认
if (-not $Force) {
    $confirm = Read-Host "确认删除以上镜像? (y/N)"
    if ([string]::IsNullOrEmpty($confirm) -or $confirm -ne 'y' -and $confirm -ne 'Y') {
        Write-Host "已取消删除操作" -ForegroundColor Yellow
        exit 0
    }
}

# 执行删除
if ($totalExpired -gt 0) {
    Write-Host ""
    Write-Host "开始删除镜像..." -ForegroundColor Yellow
    
    $deletedImages = 0
    $deletedTags = 0
    $failedCount = 0
    
    foreach ($expired in $expiredImages.Values) {
        $imageId = $expired.ID
        $tagCount = $expired.TagCount
        
        Write-Host ""
        Write-Host "处理镜像: $imageId" -ForegroundColor Cyan
        
        if ($tagCount -gt 1) {
            # 多 Tag 镜像
            if ($DeleteMode -eq "TagOnly") {
                # 只删除过期的 tag
                foreach ($img in $expired.Images) {
                    $fullName = "$($img.Repository):$($img.Tag)"
                    try {
                        Write-Host "  删除 tag: $fullName" -ForegroundColor Gray
                        docker rmi $fullName 2>&1 | Out-Null
                        $deletedTags++
                    }
                    catch {
                        Write-Host "  删除失败: $fullName" -ForegroundColor Red
                        $failedCount++
                    }
                }
            }
            else {
                # Auto 或 Force 模式：强制删除整个镜像
                try {
                    Write-Host "  强制删除镜像 (Tag数: $tagCount)" -ForegroundColor Yellow
                    docker rmi $imageId -f 2>&1 | Out-Null
                    $deletedImages++
                    $deletedTags += $tagCount
                }
                catch {
                    Write-Host "  删除失败: $imageId" -ForegroundColor Red
                    $failedCount++
                }
            }
        }
        else {
            # 单 Tag 镜像
            $img = $expired.Images[0]
            $fullName = "$($img.Repository):$($img.Tag)"
            
            try {
                Write-Host "  删除: $fullName" -ForegroundColor Gray
                docker rmi $fullName 2>&1 | Out-Null
                $deletedImages++
                $deletedTags++
            }
            catch {
                Write-Host "  删除失败: $fullName" -ForegroundColor Red
                $failedCount++
            }
        }
    }
    
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "删除完成!" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Cyan
    Write-Host "删除镜像数: $deletedImages" -ForegroundColor Green
    Write-Host "删除 Tag 数: $deletedTags" -ForegroundColor Green
    if ($failedCount -gt 0) {
        Write-Host "删除失败: $failedCount" -ForegroundColor Red
    }
    Write-Host "========================================" -ForegroundColor Cyan
} else {
    Write-Host ""
    Write-Host "没有需要删除的镜像" -ForegroundColor Green
}
