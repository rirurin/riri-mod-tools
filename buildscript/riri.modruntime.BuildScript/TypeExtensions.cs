using System.IO.Hashing;
using System.Text;

namespace riri.modruntime.BuildScript;

public static class TypeExtensions
{
    public static ulong ToXxh3(this string Value)
        => XxHash3.HashToUInt64(Encoding.UTF8.GetBytes(Value));
}